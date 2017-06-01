use std::fs;
use std::path;
use serde_json;

use errors::*;
use super::configuration::NLUEngineConfiguration;

const NLU_CONFIGURATION_FILENAME: &str = "trained_assistant.json";

pub trait NLUEngineConfigurationConvertible {
    fn nlu_engine_configuration(&self) -> &NLUEngineConfiguration;
    fn into_nlu_engine_configuration(self) -> NLUEngineConfiguration;
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
