use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct RuleBasedParserConfiguration {
    pub language: String,
    #[serde(rename="patterns")]
    pub regexes_per_intent: HashMap<String, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, String>,
    pub slot_names_to_entities: HashMap<String, String>,
}
