use std::ops::Range;
use std::collections::HashSet;

use errors::*;

pub mod rule_based;
pub mod probabilistic;
pub mod nlu_engine;
pub mod configuration;
mod tagging_utils;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct IntentParserResult {
    pub input: String,
    pub intent: Option<IntentClassifierResult>,
    pub slots: Option<Vec<Slot>>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: f32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Slot {
    pub value: String,
    pub range: Range<usize>,
    pub entity: String,
    pub slot_name: String
}

trait IntentParser: Send + Sync {
    fn get_intent(&self, input: &str, intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>>;
    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input: &I) -> O;
}
