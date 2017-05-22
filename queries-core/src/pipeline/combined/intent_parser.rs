use std::collections::HashMap;

use errors::*;
use super::configuration::SnipsConfiguration;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots};
use pipeline::rule_based::RuleBasedIntentParser;

pub struct SnipsIntentParser {
    parsers: Vec<Box<IntentParser>>,
}

impl SnipsIntentParser {
    pub fn new(configuration: SnipsConfiguration) -> Result<Self> {
        let rule_based_parser = RuleBasedIntentParser::new(configuration.model.rule_based_parser)?;

        Ok(SnipsIntentParser { parsers: vec![Box::new(rule_based_parser)]})
    }
}

impl IntentParser for SnipsIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        for p in self.parsers.iter() {
            let result = p.parse(input, probability_threshold)?;
            if result.is_some() {
                return Ok(result);
            }
        }
        Ok(None)
    }

    fn get_intent(&self,
                  input: &str,
                  probability_threshold: f32)
                  -> Result<Vec<IntentClassifierResult>> {
        for p in self.parsers.iter() {
            let result = p.get_intent(input, probability_threshold)?;
            if !result.is_empty() {
                return Ok(result);
            }
        }
        Ok(vec![])
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        for p in self.parsers.iter() {
            let result = p.get_entities(input, intent_name)?;
            if !result.is_empty() {
                return Ok(result);
            }
        }
        Ok(HashMap::new())
    }
}
