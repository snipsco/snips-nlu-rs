use std::collections::HashMap;

use pipeline::rule_based::RuleBasedParserConfiguration;
use pipeline::probabilistic::ProbabilisticParserConfiguration;

pub trait NluEngineConfigurationConvertible {
    fn nlu_engine_configuration(&self) -> &NluEngineConfiguration;
    fn into_nlu_engine_configuration(self) -> NluEngineConfiguration;
}

#[derive(Debug, Deserialize)]
pub struct NluEngineConfiguration {
    pub language: String,
    pub model: Model,
    pub entities: HashMap<String, Entity>,
    pub intents_data_sizes: HashMap<String, usize>,
    pub slot_name_mapping: HashMap<String, HashMap<String, String>>
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub rule_based_parser: Option<RuleBasedParserConfiguration>,
    pub probabilistic_parser: Option<ProbabilisticParserConfiguration>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entity {
    pub automatically_extensible: bool,
    pub utterances: HashMap<String, String>
}

impl NluEngineConfigurationConvertible for NluEngineConfiguration {
    fn nlu_engine_configuration(&self) -> &NluEngineConfiguration {
        &self
    }

    fn into_nlu_engine_configuration(self) -> NluEngineConfiguration {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::NluEngineConfiguration;

    use testutils::parse_json;

    #[test]
    fn deserialization_works() {
        let retrieved: NluEngineConfiguration = parse_json("tests/configurations/trained_assistant.json");
        assert_eq!("en", retrieved.model.rule_based_parser.unwrap().language_code);
        assert_eq!("en", retrieved.model.probabilistic_parser.unwrap().language_code);
    }
}
