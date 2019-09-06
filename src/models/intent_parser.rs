use std::collections::HashMap;

use serde::Deserialize;

use crate::utils::{EntityName, IntentName, SlotName};

pub type InputHash = i32;
pub type IntentId = i32;
pub type SlotId = i32;

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
    pub map: HashMap<InputHash, (IntentId, Vec<SlotId>)>,
    pub entity_scopes: Vec<GroupedEntityScope>,
    pub stop_words_whitelist: HashMap<IntentName, Vec<String>>,
    pub config: LookupParserConfig,
}

#[derive(Debug, Deserialize)]
pub struct GroupedEntityScope {
    pub intent_group: Vec<IntentName>,
    pub entity_scope: EntityScope,
}

#[derive(Debug, Deserialize)]
pub struct EntityScope {
    pub builtin: Vec<EntityName>,
    pub custom: Vec<EntityName>,
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
