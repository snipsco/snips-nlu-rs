use std::collections::HashMap;

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots, SlotValue};
use super::intent_classifier::IntentClassifier;
use super::tagger::Tagger;
use super::crf_utils;
use preprocessing::tokenize;

pub struct ProbabilisticIntentParser {
    intent_classifier: IntentClassifier,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
    taggers: HashMap<String, Tagger>,
}

impl ProbabilisticIntentParser {
    pub fn new() -> Result<Self> {
        Ok(ProbabilisticIntentParser {
               intent_classifier: IntentClassifier::new(),
               slot_name_to_entity_mapping: HashMap::new(),
               taggers: HashMap::new(),
           })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        unimplemented!()
    }

    fn get_intent(&self, input: &str, _: f32) -> Result<Vec<IntentClassifierResult>> {
        self.intent_classifier.get_intent(input)
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        let tagger = self.taggers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in taggers", intent_name))?;

        let intent_slots_mapping = self.slot_name_to_entity_mapping
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in slots name mapping", intent_name))?;

        let tokens = tokenize(input);
        if tokens.is_empty() {
            return Ok(HashMap::new());
        }

        let tags = tagger.get_tags(&tokens)?;
        let slots = crf_utils::tags_to_slots(input, &tokens, &tags, tagger.tagging_scheme, intent_slots_mapping);

        // TODO: Augment slots with builtin entities

        Ok(convert_vec_slots_to_hashmap(slots))
    }
}

fn convert_vec_slots_to_hashmap(slots: Vec<(String, SlotValue)>) -> Slots {
    const ESTIMATED_MAX_SLOT: usize = 10;
    const ESTIMATED_MAX_SLOTVALUES: usize = 5;

    slots
        .into_iter()
        .fold(HashMap::with_capacity(ESTIMATED_MAX_SLOT),
              |mut hm, (slot_name, slot_value)| {
                  hm.entry(slot_name)
                      .or_insert_with(|| Vec::with_capacity(ESTIMATED_MAX_SLOTVALUES))
                      .push(slot_value);
                  hm
        })
}
