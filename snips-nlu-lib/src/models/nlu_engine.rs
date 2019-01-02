use std::collections::HashMap;

use crate::utils::{EntityName, IntentName, SlotName};

#[derive(Debug, Deserialize)]
pub struct ModelVersion {
    pub model_version: String,
}

#[derive(Debug, Deserialize)]
pub struct NluEngineModel {
    pub dataset_metadata: DatasetMetadata,
    pub intent_parsers: Vec<String>,
    pub model_version: String,
    pub training_package_version: String,
    pub builtin_entity_parser: String,
    pub custom_entity_parser: String,
}

#[derive(Debug, Deserialize)]
pub struct DatasetMetadata {
    pub language_code: String,
    pub entities: HashMap<String, Entity>,
    pub slot_name_mappings: HashMap<IntentName, HashMap<SlotName, EntityName>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entity {
    pub automatically_extensible: bool,
}
