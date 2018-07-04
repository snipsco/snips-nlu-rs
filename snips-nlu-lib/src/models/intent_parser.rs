use std::collections::HashMap;

use models::{IntentClassifierModel, SlotFillerModel};

#[derive(Debug, Deserialize)]
pub struct DeterministicParserModel {
    pub language_code: String,
    pub patterns: HashMap<String, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, String>,
    pub slot_names_to_entities: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserModel {
    pub intent_classifier: IntentClassifierModel,
    pub slot_fillers: HashMap<String, SlotFillerModel>,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserModel2 {
    pub slot_fillers: Vec<SlotFillerMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct SlotFillerMetadata {
    pub intent: String,
    pub slot_filler_name: String,
}
