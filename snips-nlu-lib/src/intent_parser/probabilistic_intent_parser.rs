use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;

use errors::*;
use failure::ResultExt;
use intent_classifier::{build_intent_classifier, IntentClassifier};
use intent_parser::{IntentParser, InternalParsingResult};
use models::ProbabilisticParserModel;
use resources::SharedResources;
use serde_json;
use slot_filler::{build_slot_filler, SlotFiller};
use utils::IntentName;

pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_fillers: HashMap<IntentName, Box<SlotFiller>>,
}

impl ProbabilisticIntentParser {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(&parser_model_path)
            .with_context(|_|
                format!("Cannot open ProbabilisticIntentParser file '{:?}'",
                        &parser_model_path))?;
        let model: ProbabilisticParserModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize ProbabilisticIntentParser json data")?;
        let intent_classifier_path = path.as_ref().join("intent_classifier");
        let intent_classifier = build_intent_classifier(
            intent_classifier_path, shared_resources.clone())?;
        let slot_fillers_vec: Result<Vec<_>> = model.slot_fillers.iter()
            .map(|metadata| {
                let slot_filler_path = path.as_ref().join(&metadata.slot_filler_name);
                Ok((
                    metadata.intent.to_string(),
                    build_slot_filler(slot_filler_path, shared_resources.clone())?,
                ))
            })
            .collect();
        let slot_fillers = HashMap::from_iter(slot_fillers_vec?);
        Ok(Self { intent_classifier, slot_fillers })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(
        &self,
        input: &str,
        intents: Option<&HashSet<IntentName>>,
    ) -> Result<Option<InternalParsingResult>> {
        let opt_intent_result = self.intent_classifier.get_intent(input, intents)?;
        if let Some(intent_result) = opt_intent_result {
            let slots = self.slot_fillers
                .get(&*intent_result.intent_name)
                .ok_or_else(|| {
                    format_err!(
                        "intent {} not found in slot fillers",
                        intent_result.intent_name
                    )
                })?
                .get_slots(input)?;
            Ok(Some(InternalParsingResult {
                intent: intent_result,
                slots,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::file_path;
    use slot_utils::InternalSlot;
    use resources::loading::load_language_resources;

    #[test]
    fn from_path_works() {
        // Given
        let trained_engine_path = file_path("tests")
            .join("models")
            .join("trained_engine");

        let parser_path = trained_engine_path
            .join("probabilistic_intent_parser");

        let resources_path = trained_engine_path.join("resources").join("en");
        let resources = load_language_resources(resources_path).unwrap();

        // When
        let intent_parser = ProbabilisticIntentParser::from_path(parser_path, resources).unwrap();
        let parsing_result = intent_parser.parse("make me two cups of coffee", None).unwrap();

        // Then
        let expected_intent = Some("MakeCoffee");
        let expected_slots = Some(vec![
            InternalSlot {
                value: "two".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            }
        ]);
        assert_eq!(expected_intent, parsing_result.as_ref().map(|res| &*res.intent.intent_name));
        assert_eq!(expected_slots, parsing_result.map(|res| res.slots));
    }
}
