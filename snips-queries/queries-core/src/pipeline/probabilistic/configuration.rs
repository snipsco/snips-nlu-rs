use std::collections::HashMap;

use serde_json;

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserConfiguration {
    pub language_code: String,
    pub taggers: HashMap<String, TaggerConfiguration>,
    pub intent_classifier: IntentClassifierConfiguration,
    pub slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct IntentClassifierConfiguration {
    pub featurizer: Option<FeaturizerConfiguration>,
    pub intercept: Option<Vec<f32>>,
    pub coeffs: Option<Vec<Vec<f32>>>,
    pub intent_list: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfiguration {
    pub language_code: String,
    pub tfidf_vectorizer: TfIdfVectorizerConfiguration,
    pub config: FeaturizerConfigConfiguration,
    pub best_features: Vec<usize>,
    pub entity_utterances_to_feature_names: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfigConfiguration {
    pub sublinear_tf: bool
}

#[derive(Debug, Deserialize)]
pub struct TfIdfVectorizerConfiguration {
    pub idf_diag: Vec<f32>,
    pub vocab: HashMap<String, usize>,
}

#[derive(Debug, Deserialize)]
pub struct TaggerConfiguration {
    pub crf_model_data: String,
    pub tagging_scheme: u8,
    pub features_signatures: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    pub factory_name: String,
    pub offsets: Vec<i32>,
    pub args: HashMap<String, serde_json::Value>,
}
