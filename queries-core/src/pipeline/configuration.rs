use std::collections::HashMap;
use std::fs;
use std::path;
use serde_json;

use errors::*;

use pipeline::rule_based::RuleBasedParserConfiguration;
use pipeline::probabilistic::ProbabilisticParserConfiguration;

const NLU_CONFIGURATION_FILENAME: &str = "trained_assistant.json";

pub trait NLUEngineConfigurationConvertible {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration;
    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration;
}

#[derive(Debug, Deserialize)]
pub struct NLUEngineConfiguration {
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

pub struct FileBasedConfiguration {
    nlu_configuration: NLUEngineConfiguration,
}

impl FileBasedConfiguration {
    pub fn new<P: AsRef<path::Path>>(root_dir: P) -> Result<Self> {
        let config_file = fs::File::open(root_dir.as_ref().join(NLU_CONFIGURATION_FILENAME))?;

        Ok(Self { nlu_configuration: serde_json::from_reader(config_file)? })
    }
}

impl NLUEngineConfigurationConvertible for FileBasedConfiguration {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration {
        &self.nlu_configuration
    }

    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration {
        self.nlu_configuration
    }
}

impl NLUEngineConfigurationConvertible for NLUEngineConfiguration {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration {
        &self
    }

    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::NLUEngineConfiguration;

    use utils::miscellaneous::parse_json;

    #[test]
    fn deserialization_works() {
        let retrieved: NLUEngineConfiguration = parse_json("tests/configurations/beverage_engine.json");
        assert_eq!("en", retrieved.model.rule_based_parser.unwrap().language);
        assert_eq!("en", retrieved.model.probabilistic_parser.unwrap().language_code);
    }
}
