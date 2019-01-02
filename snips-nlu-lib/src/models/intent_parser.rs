use std::collections::HashMap;

use crate::utils::{EntityName, IntentName, SlotName};

#[derive(Debug, Deserialize)]
pub struct DeterministicParserModel {
    pub language_code: String,
    pub patterns: HashMap<IntentName, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, SlotName>,
    pub slot_names_to_entities: HashMap<IntentName, HashMap<SlotName, EntityName>>,
    pub config: DeterministicParserConfig,
}

#[derive(Debug, Deserialize)]
pub struct DeterministicParserConfig {
    #[serde(default)]
    pub ignore_stop_words: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserModel {
    pub slot_fillers: Vec<SlotFillerMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct SlotFillerMetadata {
    pub intent: IntentName,
    pub slot_filler_name: String,
}
