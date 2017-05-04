use std::collections::HashMap;

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots, SlotValue};

pub struct ProbabilisticIntentParser {
}

impl ProbabilisticIntentParser {
    pub fn new() -> Result<Self> {
        Ok(ProbabilisticIntentParser {})
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        unimplemented!()
    }

    fn get_intent(&self, input: &str, probability_threshold: f32) -> Result<Vec<IntentClassifierResult>> {
        unimplemented!()
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        unimplemented!()
    }
}
