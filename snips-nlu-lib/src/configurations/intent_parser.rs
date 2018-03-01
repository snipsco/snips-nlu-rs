use std::collections::HashMap;

use super::{IntentClassifierConfiguration, SlotFillerConfiguration};


#[derive(Debug, Deserialize)]
pub struct DeterministicParserConfiguration {
    pub language_code: String,
    pub patterns: HashMap<String, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, String>,
    pub slot_names_to_entities: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserConfiguration {
    pub intent_classifier: IntentClassifierConfiguration,
    pub slot_fillers: HashMap<String, SlotFillerConfiguration>,
}
