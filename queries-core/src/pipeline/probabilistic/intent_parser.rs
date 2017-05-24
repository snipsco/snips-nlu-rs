use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, Slot};
use super::intent_classifier::IntentClassifier;
use super::tagger::Tagger;
use super::crf_utils;
use preprocessing::tokenize;
use super::configuration::ProbabilisticParserConfiguration;

pub struct ProbabilisticIntentParser {
    intent_classifier: IntentClassifier,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
    taggers: HashMap<String, Tagger>,
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserConfiguration) -> Result<Self> {
        let taggers: Result<Vec<_>> = config.taggers.into_iter()
            .map(|(intent_name, tagger_config)| Ok((intent_name, Tagger::new(tagger_config)?)))
            .collect();

        let taggers_map: HashMap<String, Tagger> = HashMap::from_iter(taggers?);
        let intent_classifier = IntentClassifier::new(config.intent_classifier)?;
        Ok(ProbabilisticIntentParser {
            intent_classifier,
            slot_name_to_entity_mapping: config.slot_name_to_entity_mapping,
            taggers: taggers_map,
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn get_intent(&self, input: &str,
                  intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>> {
        if let Some(intents_set) = intents {
            if intents_set.len() == 1 {
                Ok(Some(
                    IntentClassifierResult {
                        intent_name: intents_set.into_iter().next().unwrap().to_string(),
                        probability: 1.0
                    }
                ))
            } else {
                let result = self.intent_classifier.get_intent(input)?;
                if let Some(res) = result {
                    if intents_set.contains(&res.intent_name) {
                        Ok(Some(res))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(result)
                }
            }
        } else {
            self.intent_classifier.get_intent(input)
        }
    }

    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>> {
        let tagger = self.taggers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in taggers", intent_name))?;

        let intent_slots_mapping = self.slot_name_to_entity_mapping
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in slots name mapping", intent_name))?;

        let tokens = tokenize(input);
        if tokens.is_empty() {
            return Ok(vec![]);
        }

        let tags = tagger.get_tags(&tokens)?;
        let slots = crf_utils::tags_to_slots(input, &tokens, &tags, tagger.tagging_scheme, intent_slots_mapping);

        // TODO: Augment slots with builtin entities

        Ok(slots)
    }
}
