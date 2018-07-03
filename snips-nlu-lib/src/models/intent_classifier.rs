use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct IntentClassifierModel {
    pub featurizer: Option<FeaturizerModel>,
    pub intercept: Option<Vec<f32>>,
    pub coeffs: Option<Vec<Vec<f32>>>,
    pub intent_list: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerModel {
    pub language_code: String,
    pub tfidf_vectorizer: TfIdfVectorizerModel,
    pub config: FeaturizerConfiguration,
    pub best_features: Vec<usize>,
    pub entity_utterances_to_feature_names: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfiguration {
    pub sublinear_tf: bool,
    pub word_clusters_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TfIdfVectorizerModel {
    pub idf_diag: Vec<f32>,
    pub vocab: HashMap<String, usize>,
}
