use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DeterministicParserConfiguration {
    pub language_code: String,
    pub patterns: HashMap<String, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, String>,
    pub slot_names_to_entities: HashMap<String, String>,
}
