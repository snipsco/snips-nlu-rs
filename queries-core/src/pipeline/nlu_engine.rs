use std::collections::HashMap;

use errors::*;
use super::configuration::SnipsConfiguration;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slot};
use pipeline::rule_based::RuleBasedIntentParser;
use pipeline::probabilistic::ProbabilisticIntentParser;
use super::configuration::Entity;

pub struct SnipsNLUEngine {
    parsers: Vec<Box<IntentParser>>,
    entities: HashMap<String, Entity>
}

impl SnipsNLUEngine {
    pub fn new(configuration: SnipsConfiguration) -> Result<Self> {
        let model = configuration.model;
        let rule_based_parser =
            if let Some(config) = model.rule_based_parser {
                Some(RuleBasedIntentParser::new(config)?)
            } else {
                None
            };
        let probabilistic_parser =
            if let Some(config) = model.probabilistic_parser {
                Some(ProbabilisticIntentParser::new(config)?)
            } else {
                None
            };

        let parsers: Vec<Box<IntentParser>> = vec![
            rule_based_parser.map(|p| Box::new(p) as _),
            probabilistic_parser.map(|p| Box::new(p) as _)
        ].into_iter().filter_map(|p| p).collect();

        Ok(SnipsNLUEngine { parsers, entities: configuration.entities })
    }

    pub fn parse(&self, input: &str, intent: Option<&str>) -> Result<IntentParserResult> {
        if self.parsers.is_empty() {
            return Ok(IntentParserResult { input: input.to_string(), intent: None, slots: None });
        }
        for parser in self.parsers.iter() {
            let classification_result =
                if let Some(intent_name) = intent {
                    Some(IntentClassifierResult {
                        intent_name: intent_name.to_string(),
                        probability: 1.0
                    })
                } else {
                    parser.get_intent(input)?
                };
            if let Some(classification_result) = classification_result {
                let valid_slots = parser
                    .get_slots(input, &classification_result.intent_name)?
                    .into_iter()
                    .filter_map(|s| {
                        let mut slot_value = s.value.to_string();
                        if self.entities.contains_key(&s.entity) {
                            let entity = self.entities.get(&s.entity).unwrap();
                            if !entity.automatically_extensible {
                                if !entity.utterances.contains_key(&s.value) {
                                    return None
                                }
                                slot_value = entity.utterances.get(&s.value).unwrap().to_string();
                            }
                        };
                        Some(Slot {
                            value: slot_value,
                            range: s.range,
                            entity: s.entity,
                            slot_name: s.slot_name
                        })
                    })
                    .collect();

                return Ok(
                    IntentParserResult {
                        input: input.to_string(),
                        intent: Some(classification_result),
                        slots: Some(valid_slots)
                    }
                )
            }
        }

        Ok(IntentParserResult { input: input.to_string(), intent: None, slots: None })
    }
}
