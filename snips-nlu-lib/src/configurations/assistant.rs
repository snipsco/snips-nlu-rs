use std::io::{Read, Seek};
use std::sync::{Arc, Mutex};
use std::path;
use std::fs;

use failure::ResultExt;

use errors::*;
use configurations::{ModelVersionConfiguration, NluEngineConfiguration,
                     NluEngineConfigurationConvertible};

const NLU_CONFIGURATION_FILENAME: &str = "trained_assistant.json";

pub struct FileBasedConfiguration {
    nlu_configuration: NluEngineConfiguration,
}

impl FileBasedConfiguration {
    pub fn new<P: AsRef<path::Path>>(
        root_dir: P,
        bypass_model_version_check: bool,
    ) -> Result<Self> {
        let path = root_dir.as_ref().join(NLU_CONFIGURATION_FILENAME);

        if !bypass_model_version_check {
            let config_file =
                fs::File::open(&path).context(SnipsNluError::ConfigLoad(format!("{:?}", path)))?;
            let nlu_configuration_with_version_only: ModelVersionConfiguration =
                ::serde_json::from_reader(config_file)
                    .context(SnipsNluError::ConfigLoad(format!("{:?}", path)))?;
            if nlu_configuration_with_version_only.model_version
                != ::SnipsNluEngine::model_version()
            {
                bail!(SnipsNluError::WrongModelVersion(
                    nlu_configuration_with_version_only.model_version,
                    ::SnipsNluEngine::model_version(),
                ));
            }
        }

        let config_file =
            fs::File::open(&path).context(SnipsNluError::ConfigLoad(format!("{:?}", path)))?;
        let nlu_configuration = ::serde_json::from_reader(config_file)
            .context(SnipsNluError::ConfigLoad(format!("{:?}", path)))?;

        Ok(Self { nlu_configuration })
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
            .context(SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;

        if !bypass_model_version_check {
            let nlu_configuration_with_version_only: ModelVersionConfiguration =
                ::serde_json::from_slice(&nlu_conf_bytes)
                    .context(SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;
            if nlu_configuration_with_version_only.model_version
                != ::SnipsNluEngine::model_version()
            {
                bail!(SnipsNluError::WrongModelVersion(
                    nlu_configuration_with_version_only.model_version,
                    ::SnipsNluEngine::model_version(),
                ));
            }
        }

        let nlu_configuration = ::serde_json::from_slice(&nlu_conf_bytes)
            .context(SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;

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
}

pub struct BytesBasedConfiguration {
    nlu_configuration: NluEngineConfiguration,
}

impl BytesBasedConfiguration {
    pub fn new(
        nlu_conf_bytes: &[u8],
    ) -> Result<Self> {
        let nlu_configuration = ::serde_json::from_slice(nlu_conf_bytes)
            .context(SnipsNluError::ConfigLoad(NLU_CONFIGURATION_FILENAME.into()))?;

        Ok(Self { nlu_configuration })
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
    use std::fs;
    use super::*;
    use utils::file_path;

    #[test]
    fn file_based_assistant_works() {
        let file = file_path("tests/configurations");
        let nlu_config_formatted = FileBasedConfiguration::new(file, false)
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

    #[test]
    fn bytes_based_assistant_works() {

        let mut f = fs::File::open("examples/trained_assistant.json").expect("file not found");

        let mut config_string = String::new();
            f.read_to_string(&mut config_string)
            .expect("something went wrong reading the file");

        let nlu_config_formatted = BytesBasedConfiguration::new(config_string.as_bytes())
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_config_formatted);
    }
}
