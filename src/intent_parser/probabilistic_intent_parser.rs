use std::collections::HashMap;
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;

use failure::{format_err, ResultExt};

use crate::errors::*;
use crate::intent_classifier::{build_intent_classifier, IntentClassifier};
use crate::models::ProbabilisticParserModel;
use crate::resources::SharedResources;
use crate::slot_filler::{build_slot_filler, SlotFiller};
use crate::utils::IntentName;

use super::{IntentClassifierResult, IntentParser, InternalParsingResult};
use crate::slot_utils::InternalSlot;

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
        let model_file = File::open(&parser_model_path).with_context(|_| {
            format!(
                "Cannot open ProbabilisticIntentParser file '{:?}'",
                &parser_model_path
            )
        })?;
        let model: ProbabilisticParserModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize ProbabilisticIntentParser json data")?;
        let intent_classifier_path = path.as_ref().join("intent_classifier");
        let intent_classifier =
            build_intent_classifier(intent_classifier_path, shared_resources.clone())?;
        let slot_fillers_vec: Result<Vec<_>> = model
            .slot_fillers
            .iter()
            .map(|metadata| {
                let slot_filler_path = path.as_ref().join(&metadata.slot_filler_name);
                Ok((
                    metadata.intent.to_string(),
                    build_slot_filler(slot_filler_path, shared_resources.clone())?,
                ))
            })
            .collect();
        let slot_fillers = HashMap::from_iter(slot_fillers_vec?);
        Ok(Self {
            intent_classifier,
            slot_fillers,
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(
        &self,
        input: &str,
        intents_whitelist: Option<&[&str]>,
    ) -> Result<InternalParsingResult> {
        let intent_result = self
            .intent_classifier
            .get_intent(input, intents_whitelist)?;
        let slots = if let Some(name) = intent_result.intent_name.as_ref() {
            self.slot_fillers
                .get(name)
                .ok_or_else(|| SnipsNluError::UnknownIntent(name.to_string()))?
                .get_slots(input)?
        } else {
            vec![]
        };
        Ok(InternalParsingResult {
            intent: intent_result,
            slots,
        })
    }

    fn get_intents(&self, input: &str) -> Result<Vec<IntentClassifierResult>> {
        self.intent_classifier.get_intents(input)
    }

    fn get_slots(&self, input: &str, intent: &str) -> Result<Vec<InternalSlot>> {
        self.slot_fillers
            .get(intent)
            .ok_or_else(|| format_err!("Unknown intent: {}", intent))
            .and_then(|slot_filler| slot_filler.get_slots(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::loading::load_engine_shared_resources;
    use crate::slot_utils::InternalSlot;

    #[test]
    fn test_parse() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let parser_path = trained_engine_path.join("probabilistic_intent_parser");
        let resources = load_engine_shared_resources(trained_engine_path).unwrap();

        // When
        let intent_parser = ProbabilisticIntentParser::from_path(parser_path, resources).unwrap();
        let parsing_result = intent_parser
            .parse("make me two cups of coffee", None)
            .unwrap();

        // Then
        let expected_intent = Some("MakeCoffee".to_string());
        let expected_slots = vec![InternalSlot {
            value: "two".to_string(),
            char_range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
        }];
        assert_eq!(expected_intent, parsing_result.intent.intent_name);
        assert_eq!(expected_slots, parsing_result.slots);
    }

    #[test]
    fn test_get_slots() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let parser_path = trained_engine_path.join("probabilistic_intent_parser");
        let resources = load_engine_shared_resources(trained_engine_path).unwrap();

        // When
        let intent_parser = ProbabilisticIntentParser::from_path(parser_path, resources).unwrap();
        let slots = intent_parser
            .get_slots("make me two hot cups of tea", "MakeTea")
            .unwrap();

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "two".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            },
            InternalSlot {
                value: "hot".to_string(),
                char_range: 12..15,
                entity: "Temperature".to_string(),
                slot_name: "beverage_temperature".to_string(),
            },
        ];
        assert_eq!(expected_slots, slots);
    }
}
