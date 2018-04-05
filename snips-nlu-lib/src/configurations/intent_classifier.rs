use std::collections::HashMap;

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
    pub sublinear_tf: bool,
    pub word_clusters_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TfIdfVectorizerConfiguration {
    pub idf_diag: Vec<f32>,
    pub vocab: HashMap<String, usize>,
}
