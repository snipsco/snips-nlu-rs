use std::collections::HashMap;

pub trait NluEngineModelConvertible {
    fn nlu_engine_model(&self) -> &NluEngineModel;
    fn into_nlu_engine_model(self) -> NluEngineModel;
}

#[derive(Debug, Deserialize)]
pub struct ModelVersion {
    pub model_version: String,
}

#[derive(Debug, Deserialize)]
pub struct NluEngineModel {
    pub dataset_metadata: DatasetMetadata,
    pub intent_parsers: Vec<String>,
    pub model_version: String,
    pub training_package_version: String,
}

#[derive(Debug, Deserialize)]
pub struct DatasetMetadata {
    pub language_code: String,
    pub entities: HashMap<String, Entity>,
    pub slot_name_mappings: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entity {
    pub automatically_extensible: bool,
    pub utterances: HashMap<String, String>,
}

impl NluEngineModelConvertible for NluEngineModel {
    fn nlu_engine_model(&self) -> &NluEngineModel {
        self
    }

    fn into_nlu_engine_model(self) -> NluEngineModel {
        self
    }
}
