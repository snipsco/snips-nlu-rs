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
    pub intent_parsers: Vec<::serde_json::Value>,
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

#[cfg(test)]
mod tests {
    use super::NluEngineModel;
    use MODEL_VERSION;
    use models::{DeterministicParserModel, ProbabilisticParserModel};

    use testutils::parse_json;

    #[test]
    fn deserialization_works() {
        // When
        let retrieved: NluEngineModel =
            parse_json("tests/models/trained_assistant.json");
        let deterministic_parser_config: Result<DeterministicParserModel, _> =
            ::serde_json::from_value(retrieved.intent_parsers[0].clone());
        let proba_parser_config: Result<ProbabilisticParserModel, _> =
            ::serde_json::from_value(retrieved.intent_parsers[1].clone());

        // Then
        let deterministic_parser_config_formatted = deterministic_parser_config
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));
        let proba_parser_formatted = proba_parser_config
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!(MODEL_VERSION, retrieved.model_version);
        assert_eq!(2, retrieved.intent_parsers.len());
        assert_eq!(Ok("ok"), deterministic_parser_config_formatted);
        assert_eq!(Ok("ok"), proba_parser_formatted);
    }
}
