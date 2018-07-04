use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;

use errors::*;
use intent_classifier::{build_intent_classifier, IntentClassifier};
use intent_parser::{IntentParser, InternalParsingResult};
use models::{FromPath, ProbabilisticParserModel};
use serde_json;
use slot_filler::{build_slot_filler, SlotFiller};

pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_fillers: HashMap<String, Box<SlotFiller>>,
}

impl FromPath for ProbabilisticIntentParser {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(parser_model_path)?;
        let model: ProbabilisticParserModel = serde_json::from_reader(model_file)?;
        let intent_classifier_path = path.as_ref().join("intent_classifier");
        let intent_classifier = build_intent_classifier(intent_classifier_path)?;
        let slot_fillers_vec: Result<Vec<_>> = model.slot_fillers.iter()
            .map(|metadata|
                Ok((
                    metadata.intent.to_string(),
                    build_slot_filler(path.as_ref().join(&metadata.slot_filler_name))?,
                ))
            )
            .collect();
        let slot_fillers = HashMap::from_iter(slot_fillers_vec?);
        Ok(Self { intent_classifier, slot_fillers })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(
        &self,
        input: &str,
        intents: Option<&HashSet<String>>,
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

    #[test]
    fn from_path_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("trained_engine")
            .join("probabilistic_intent_parser");

        // When
        let intent_parser = ProbabilisticIntentParser::from_path(path).unwrap();
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
