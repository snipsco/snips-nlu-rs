use std::fs;
use std::io::{Read, Seek};
use std::path;
use std::sync::{Arc, Mutex};

use failure::ResultExt;

use models::{ModelVersion, NluEngineModel, NluEngineModelConvertible};
use errors::*;

const NLU_MODEL_FILENAME: &str = "trained_assistant.json";

pub struct FileBasedModel {
    nlu_engine_model: NluEngineModel,
}

impl FileBasedModel {
    pub fn from_dir<P: AsRef<path::Path>>(
        root_dir: P,
        bypass_model_version_check: bool,
    ) -> Result<Self> {
        Self::from_path(
            root_dir.as_ref().join(NLU_MODEL_FILENAME),
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
                .with_context(|_| SnipsNluError::ModelLoad(path.to_str().unwrap().to_string()))?;
        }

        let model_file = fs::File::open(&path)
            .with_context(|_| SnipsNluError::ModelLoad(path.to_str().unwrap().to_string()))?;
        let nlu_engine_model = ::serde_json::from_reader(model_file)
            .with_context(|_| SnipsNluError::ModelLoad(path.to_str().unwrap().to_string()))?;

        Ok(Self { nlu_engine_model })
    }

    fn check_model_version<P: AsRef<path::Path>>(path: P) -> Result<()> {
        let model_file = fs::File::open(&path)?;

        let model_version: ModelVersion = ::serde_json::from_reader(model_file)?;
        if model_version.model_version != ::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion(
                model_version.model_version,
                ::MODEL_VERSION
            ));
        }
        Ok(())
    }
}

impl NluEngineModelConvertible for FileBasedModel {
    fn nlu_engine_model(&self) -> &NluEngineModel {
        &self.nlu_engine_model
    }

    fn into_nlu_engine_model(self) -> NluEngineModel {
        self.nlu_engine_model
    }
}

pub struct ZipBasedModel {
    nlu_engine_model: NluEngineModel,
}

impl ZipBasedModel {
    pub fn new<R>(reader: R, bypass_model_version_check: bool) -> Result<Self>
        where
            R: Read + Seek,
    {
        let zip = ::zip::ZipArchive::new(reader).context("Could not load ZipBasedModel")?;
        let mutex = Arc::new(Mutex::new(zip));

        let nlu_model_bytes = Self::read_bytes(&mutex, NLU_MODEL_FILENAME)
            .or_else(|_| {
                // Assistants downloaded from the console are in a directory named assistant
                Self::read_bytes(&mutex, &format!("assistant/{}", NLU_MODEL_FILENAME))
            })
            .with_context(|_| SnipsNluError::ModelLoad(NLU_MODEL_FILENAME.into()))?;

        if !bypass_model_version_check {
            Self::check_model_version(&nlu_model_bytes)
                .with_context(|_| SnipsNluError::ModelLoad(NLU_MODEL_FILENAME.into()))?;
        }

        let nlu_engine_model = ::serde_json::from_slice(&nlu_model_bytes)
            .with_context(|_| SnipsNluError::ModelLoad(NLU_MODEL_FILENAME.into()))?;

        Ok(Self { nlu_engine_model })
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

    fn check_model_version(nlu_model_bytes: &[u8]) -> Result<()> {
        let model_version: ModelVersion = ::serde_json::from_slice(nlu_model_bytes)?;
        if model_version.model_version != ::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion(
                model_version.model_version,
                ::MODEL_VERSION
            ));
        }
        Ok(())
    }
}

impl NluEngineModelConvertible for ZipBasedModel {
    fn nlu_engine_model(&self) -> &NluEngineModel {
        &self.nlu_engine_model
    }

    fn into_nlu_engine_model(self) -> NluEngineModel {
        self.nlu_engine_model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use utils::file_path;

    #[test]
    fn file_based_assistant_works() {
        let file = file_path("tests/models/trained_assistant.json");
        let nlu_model_formatted = FileBasedModel::from_path(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_model_formatted);
    }

    #[test]
    fn dir_based_assistant_works() {
        let file = file_path("tests/models");
        let nlu_model_formatted = FileBasedModel::from_dir(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_model_formatted);
    }

    #[test]
    fn zip_based_assistant_works() {
        let file = fs::File::open(file_path("tests/zip_files/sample_config.zip")).unwrap();
        let nlu_model_formatted = ZipBasedModel::new(file, false)
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(Ok("ok"), nlu_model_formatted);
    }
}
