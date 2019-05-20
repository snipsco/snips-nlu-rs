use std::collections::HashMap;

use serde_derive::Deserialize;

use crate::utils::{EntityName, IntentName, SlotName};

#[derive(Debug, Deserialize)]
pub struct DeterministicParserModel {
    pub language_code: String,
    pub patterns: HashMap<IntentName, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, SlotName>,
    pub slot_names_to_entities: HashMap<IntentName, HashMap<SlotName, EntityName>>,
    #[serde(default)]
    pub stop_words_whitelist: HashMap<IntentName, Vec<String>>,
    pub config: DeterministicParserConfig,
}

#[derive(Debug, Deserialize)]
pub struct LookupParserModel {
    pub language_code: String,
    pub slots_names: Vec<SlotName>,
    pub intents_names: Vec<IntentName>,
    pub map: HashMap<i32, (i32, Vec<i32>)>,
    pub config: LookupParserConfig,
}

#[derive(Debug, Deserialize)]
pub struct DeterministicParserConfig {
    #[serde(default)]
    pub ignore_stop_words: bool,
}

#[derive(Debug, Deserialize)]
pub struct LookupParserConfig {
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
