use std::collections::HashMap;

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots, SlotValue};
use super::intent_classifier::IntentClassifier;
use super::tagger::Tagger;
use preprocessing::light::tokenize;

pub struct ProbabilisticIntentParser {
    intent_classifier: IntentClassifier,
    tagger_per_intent: HashMap<String, Tagger>,
}

impl ProbabilisticIntentParser {
    pub fn new() -> Result<Self> {
        Ok(ProbabilisticIntentParser {
               intent_classifier: IntentClassifier {},
               tagger_per_intent: HashMap::new(),
           })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        unimplemented!()
    }

    fn get_intent(&self, input: &str, _: f32) -> Result<Vec<IntentClassifierResult>> {
        Ok(self.intent_classifier.get_intent(input))
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        let tagger = self.tagger_per_intent
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;

        let tokens = tokenize(input);
        if tokens.is_empty() {
            return Ok(HashMap::new());
        }

        let tags = tagger.get_tags(&tokens);

        unimplemented!()
    }
}
