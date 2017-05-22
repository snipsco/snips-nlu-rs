use std::collections::HashMap;
use std::ops::Range;

use errors::*;

pub mod combined;
pub mod rule_based;
pub mod probabilistic;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct IntentParserResult {
    pub input: String,
    pub likelihood: f32,
    pub intent_name: String,
    pub slots: Slots,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: f32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SlotValue {
    pub value: String,
    pub range: Range<usize>,
    pub entity: String,
}

pub type Slots = HashMap<String, Vec<SlotValue>>;

pub trait IntentParser: Send + Sync {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>>;
    fn get_intent(&self,
                  input: &str,
                  probability_threshold: f32)
                  -> Result<Vec<IntentClassifierResult>>;
    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input: &I) -> O;
}
