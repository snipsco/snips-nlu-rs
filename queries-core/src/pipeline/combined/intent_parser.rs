use std::collections::HashMap;

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots};

pub struct CombinedIntentParser {
    parsers: Vec<Box<IntentParser>>,
}

impl CombinedIntentParser {
    pub fn new(parsers: Vec<Box<IntentParser>>) -> Self {
        CombinedIntentParser { parsers }
    }
}

impl IntentParser for CombinedIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        for p in self.parsers.iter() {
            let result = p.parse(input, probability_threshold)?;
            if result.is_some() {
                return Ok(result)
            }
        }
        Ok(None)
    }

    fn get_intent(&self, input: &str, probability_threshold: f32) -> Result<Vec<IntentClassifierResult>> {
        for p in self.parsers.iter() {
            let result = p.get_intent(input, probability_threshold)?;
            if !result.is_empty() {
                return Ok(result)
            }
        }
        Ok(vec![])
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        for p in self.parsers.iter() {
            let result = p.get_entities(input, intent_name)?;
            if !result.is_empty() {
                return Ok(result)
            }
        }
        Ok(HashMap::new())
    }
}
