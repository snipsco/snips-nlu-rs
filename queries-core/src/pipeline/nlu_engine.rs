use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use errors::*;
use super::configuration::SnipsConfiguration;
use pipeline::{IntentParser, IntentParserResult, Slot};
use pipeline::rule_based::RuleBasedIntentParser;
use pipeline::probabilistic::ProbabilisticIntentParser;
use super::configuration::Entity;

pub struct SnipsNLUEngine {
    parsers: Vec<Box<IntentParser>>,
    entities: HashMap<String, Entity>
}

impl SnipsNLUEngine {
    pub fn new(configuration: SnipsConfiguration) -> Result<Self> {
        let mut parsers: Vec<Box<IntentParser>> = Vec::with_capacity(2);
        let model = configuration.model;
        if let Some(config) = model.rule_based_parser {
            parsers.push(Box::new(RuleBasedIntentParser::new(config)?))
        };
        if let Some(config) = model.probabilistic_parser {
            parsers.push(Box::new(ProbabilisticIntentParser::new(config)?))
        };
        Ok(SnipsNLUEngine { parsers, entities: configuration.entities })
    }

    pub fn parse(&self, input: &str, intents_filter: Option<&[&str]>) -> Result<IntentParserResult> {
        if self.parsers.is_empty() {
            return Ok(IntentParserResult { input: input.to_string(), intent: None, slots: None });
        }
        let set_intents: Option<HashSet<String>> = intents_filter.map(|intent_list|
            HashSet::from_iter(intent_list.iter().map(|name| name.to_string()))
        );

        for parser in self.parsers.iter() {
            let classification_result = parser.get_intent(input, set_intents.as_ref())?;
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
