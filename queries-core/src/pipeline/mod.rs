use std::ops::Range;
use std::collections::HashSet;

use errors::*;
use builtin_entities::BuiltinEntity;

pub mod rule_based;
pub mod probabilistic;
pub mod nlu_engine;
pub mod assistant_config;
pub mod configuration;
pub mod slot_utils;
mod tagging_utils;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct IntentParserResult {
    pub input: String,
    pub intent: Option<IntentClassifierResult>,
    pub slots: Option<Vec<Slot>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalSlot {
    pub value: String,
    pub range: Range<usize>,
    pub entity: String,
    pub slot_name: String
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub raw_value: String,
    pub value: SlotValue,
    pub range: Option<Range<usize>>,
    pub entity: String,
    pub slot_name: String
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content="value")]
pub enum SlotValue {
    Custom(String),
    Builtin(BuiltinEntity),
}

trait IntentParser: Send + Sync {
    fn get_intent(&self, input: &str, intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>>;
    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input: &I) -> O;
}
