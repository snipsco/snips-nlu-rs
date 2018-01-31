use std::collections::HashMap;
use serde_json;

pub trait NluEngineConfigurationConvertible {
    fn nlu_engine_configuration(&self) -> &NluEngineConfiguration;
    fn into_nlu_engine_configuration(self) -> NluEngineConfiguration;
}

#[derive(Debug, Deserialize)]
pub struct ModelVersionConfiguration {
    pub model_version: String,
}

#[derive(Debug, Deserialize)]
pub struct NluEngineConfiguration {
    pub dataset_metadata: DatasetMetadata,
    pub intent_parsers: Vec<serde_json::Value>,
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
    use pipeline::deterministic::DeterministicParserConfiguration;
    use pipeline::probabilistic::ProbabilisticParserConfiguration;

    use testutils::parse_json;
    use serde_json;

    #[test]
    fn deserialization_works() {
        // When
        let retrieved: NluEngineConfiguration = parse_json("tests/configurations/trained_assistant.json");
        let deterministic_parser_config: Result<DeterministicParserConfiguration, _> =
            serde_json::from_value(retrieved.intent_parsers[0].clone());
        let proba_parser_config: Result<ProbabilisticParserConfiguration, _> =
            serde_json::from_value(retrieved.intent_parsers[1].clone());

        // Then
        let deterministic_parser_config_formatted = deterministic_parser_config
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));
        let proba_parser_formatted = proba_parser_config
            .map(|_| "ok")
            .map_err(|err| format!("{:?}", err));

        assert_eq!("0.12.0", retrieved.model_version);
        assert_eq!(2, retrieved.intent_parsers.len());
        assert_eq!(Ok("ok"), deterministic_parser_config_formatted);
        assert_eq!(Ok("ok"), proba_parser_formatted);
    }
}
