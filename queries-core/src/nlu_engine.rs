use std::collections::HashMap;

use config::{AssistantConfig, FileBasedAssistantConfig};
use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots};
use pipeline::{deep, light};

pub struct NLUEngine {
    config: Box<AssistantConfig>,
    parsers: Vec<Box<IntentParser>>,
}

impl NLUEngine {
    pub fn new(config: Box<AssistantConfig>) -> Result<Self> {
        Ok(NLUEngine { parsers: init_parsers(&*config), config })
    }
}

fn init_parsers(config: &AssistantConfig) -> Vec<Box<IntentParser>> {
    vec![
        Box::new(deep::IntentParser::new(&*config).unwrap()),
        Box::new(light::IntentParser::new(HashMap::new(), HashMap::new(), HashMap::new()).unwrap()),
    ]
}

impl IntentParser for NLUEngine {
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
