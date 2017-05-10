use std::collections::HashMap;
use std::iter::FromIterator;

use protos::PBRegexIntentParserConfiguration;

pub struct RegexIntentParserConfiguration {
    pub language: String,
    pub regexes_per_intent: HashMap<String, Vec<String>>,
    pub group_names_to_slot_names: HashMap<String, String>,
    pub slot_names_to_entities: HashMap<String, String>,
}

impl From<PBRegexIntentParserConfiguration> for RegexIntentParserConfiguration {
    fn from(pb_config: PBRegexIntentParserConfiguration) -> RegexIntentParserConfiguration {
        let mut pb_config = pb_config;

        let regexes_per_intent_iter = pb_config
            .take_models()
            .into_iter()
            .map(|model| {
                 let patterns = model.get_patterns().to_vec();
                 (model.intent_name, patterns)
            });

        let slot_names_to_entities_iter = pb_config
            .take_slot_names_to_entities()
            .into_iter()
            .map(|(slot_name, entity)| (slot_name, entity.name));

        RegexIntentParserConfiguration {
            language: pb_config.take_language(),
            regexes_per_intent: HashMap::from_iter(regexes_per_intent_iter),
            group_names_to_slot_names: pb_config.take_group_names_to_slot_names(),
            slot_names_to_entities: HashMap::from_iter(slot_names_to_entities_iter),
        }
    }
}
