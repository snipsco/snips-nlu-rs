use std::fs;
use std::io::{Read, Seek};
use std::path;
use std::sync::{Arc, Mutex};

use failure::ResultExt;

use configurations::{ModelVersionConfiguration, NluEngineConfiguration,
                     NluEngineConfigurationConvertible};
use errors::*;

const NLU_CONFIGURATION_FILENAME: &str = "trained_assistant.json";

pub struct FileBasedConfiguration {
    nlu_configuration: NluEngineConfiguration,
}

impl FileBasedConfiguration {
    pub fn from_dir<P: AsRef<path::Path>>(
        root_dir: P,
        bypass_model_version_check: bool,
    ) -> Result<Self> {
        Self::from_path(
            root_dir.as_ref().join(NLU_CONFIGURATION_FILENAME),
            bypass_model_version_check,
        )
    }

    pub fn from_path<P: AsRef<path::Path>>(
        file_path: P,
        bypass_model_version_check: bool,
    ) -> Result<Self> {
        let path = file_path.as_ref();

        if !bypass_model_version_check {
            Self::check_model_version(&path)
                .with_context(|_| SnipsNluError::ConfigLoad(path.to_str().unwrap().to_string()))?;
        }

        let config_file = fs::File::open(&path)
            .with_context(|_| SnipsNluError::ConfigLoad(path.to_str().unwrap().to_string()))?;
        let nlu_configuration = ::serde_json::from_reader(config_file)
            .with_context(|_| SnipsNluError::ConfigLoad(path.to_str().unwrap().to_string()))?;

        Ok(Self { nlu_configuration })
    }

    fn check_model_version<P: AsRef<path::Path>>(path: P) -> Result<()> {
        let config_file = fs::File::open(&path)?;

        let config: ModelVersionConfiguration = ::serde_json::from_reader(config_file)?;
        if config.model_version != ::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion(
                config.model_version,
                ::MODEL_VERSION
            ));
        }
        Ok(())
    }
}

impl NluEngineConfigurationConvertible for FileBasedConfiguration {
    fn nlu_engine_configuration(&self) -> &NluEngineConfiguration {
        &self.nlu_configuration
    }

    fn into_nlu_engine_configuration(self) -> NluEngineConfiguration {
        self.nlu_configuration
    }
}

pub struct ZipBasedConfiguration {
    nlu_configuration: NluEngineConfiguration,
}

impl ZipBasedConfiguration {
    pub fn new<R>(reader: R, bypass_model_version_check: bool) -> Result<Self>
    where
        R: Read + Seek,
    {
        let zip = ::zip::ZipArchive::new(reader).context("Could not load ZipBasedConfiguration")?;
        let mutex = Arc::new(Mutex::new(zip));

        let nlu_conf_bytes = Self::read_bytes(&mutex, NLU_CONFIGURATION_FILENAME)
            .or_else(|_| {
                // Assistants downloaded from the console are in a directory named assistant
                Self::read_bytes(&mutex, &format!("assistant/{}", NLU_CONFIGURATION_FILENAME))
            })
            .with_context(|_| SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;

        if !bypass_model_version_check {
            Self::check_model_version(&nlu_conf_bytes)
                .with_context(|_| SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;
        }

        let nlu_configuration = ::serde_json::from_slice(&nlu_conf_bytes)
            .with_context(|_| SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;

        Ok(Self { nlu_configuration })
    }

    fn read_bytes<R>(zip: &Mutex<::zip::read::ZipArchive<R>>, name: &str) -> Result<Vec<u8>>
    where
        R: Read + Seek,
    {
        let mut locked = zip.lock()
            .map_err(|e| format_err!("Can't lock zip file: {}", e))?;
        let zip = &mut (*locked);
        let mut file = zip.by_name(name)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn check_model_version(nlu_conf_bytes: &[u8]) -> Result<()> {
        let config: ModelVersionConfiguration = ::serde_json::from_slice(nlu_conf_bytes)?;
        if config.model_version != ::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion(
                config.model_version,
                ::MODEL_VERSION
            ));
        }
        Ok(())
    }
}

impl NluEngineConfigurationConvertible for ZipBasedConfiguration {
    fn nlu_engine_configuration(&self) -> &NluEngineConfiguration {
        &self.nlu_configuration
    }

    fn into_nlu_engine_configuration(self) -> NluEngineConfiguration {
        self.nlu_configuration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use utils::file_path;

    #[test]
    fn file_based_assistant_works() {
        let file = file_path("tests/configurations/trained_assistant.json");
        let nlu_config_formatted = FileBasedConfiguration::from_path(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_config_formatted);
    }

    #[test]
    fn dir_based_assistant_works() {
        let file = file_path("tests/configurations");
        let nlu_config_formatted = FileBasedConfiguration::from_dir(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_config_formatted);
    }

    #[test]
    fn zip_based_assistant_works() {
        let file = fs::File::open(file_path("tests/zip_files/sample_config.zip")).unwrap();
        let nlu_config_formatted = ZipBasedConfiguration::new(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_config_formatted);
    }
}
