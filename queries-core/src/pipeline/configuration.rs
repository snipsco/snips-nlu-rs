use std::collections::HashMap;
use std::fs;
use std::path;
use std::io::{Read, Seek};
use std::sync::{Arc, Mutex};

use serde_json;
use zip;

use errors::*;
use pipeline::rule_based::RuleBasedParserConfiguration;
use pipeline::probabilistic::ProbabilisticParserConfiguration;

const NLU_CONFIGURATION_FILENAME: &str = "trained_assistant.json";

pub trait NLUEngineConfigurationConvertible {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration;
    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration;
}

// -

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

impl NLUEngineConfigurationConvertible for NLUEngineConfiguration {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration {
        &self
    }

    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration {
        self
    }
}

// -

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

// -

pub struct BinaryBasedConfiguration {
    nlu_configuration: NLUEngineConfiguration,
}

impl BinaryBasedConfiguration {
    pub fn new<R>(reader: R) -> Result<BinaryBasedConfiguration>
    where R: Read + Seek {
        let zip = zip::ZipArchive::new(reader)?;
        let mutex = Arc::new(Mutex::new(zip));

        let nlu_conf_bytes = Self::read_bytes(mutex.clone(), NLU_CONFIGURATION_FILENAME)?;
        Ok(Self {
            nlu_configuration: serde_json::from_slice(&nlu_conf_bytes)?,
        })
    }

    fn read_bytes<R>(zip: Arc<Mutex<zip::read::ZipArchive<R>>>, name: &str) -> Result<Vec<u8>>
    where R: Read + Seek {
        let mut locked = zip.lock()?;
        let ref mut zip = *locked;
        let mut file = zip.by_name(name)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

impl NLUEngineConfigurationConvertible for BinaryBasedConfiguration {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration {
        &self.nlu_configuration
    }

    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration {
        self.nlu_configuration
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::NLUEngineConfiguration;
    use super::NLUEngineConfigurationConvertible;
    use super::BinaryBasedConfiguration;

    use utils::miscellaneous::parse_json;
    use utils::miscellaneous::file_path;

    #[test]
    fn deserialization_works() {
        let retrieved: NLUEngineConfiguration = parse_json("tests/configurations/beverage_engine.json");
        assert_eq!("en", retrieved.model.rule_based_parser.unwrap().language);
        assert_eq!("en", retrieved.model.probabilistic_parser.unwrap().language_code);
    }

    #[test]
    fn unzip_works() {
        let file = fs::File::open(file_path("tests/zip_files/sample_config.zip")).unwrap();
        let nlu_config = BinaryBasedConfiguration::new(file)
            .unwrap()
            .into_nlu_engine_configuration();

        assert_eq!("en", nlu_config.model.rule_based_parser.unwrap().language);
        assert_eq!("en", nlu_config.model.probabilistic_parser.unwrap().language_code);
    }
}
