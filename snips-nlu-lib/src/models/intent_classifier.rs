use std::collections::HashMap;

use crate::utils::IntentName;

#[derive(Debug, Deserialize)]
pub struct IntentClassifierModel {
    pub featurizer: Option<FeaturizerModel>,
    pub intercept: Option<Vec<f32>>,
    pub coeffs: Option<Vec<Vec<f32>>>,
    pub intent_list: Vec<Option<IntentName>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerModel {
    pub language_code: String,
    pub tfidf_vectorizer: TfIdfVectorizerModel,
    pub config: FeaturizerConfiguration,
    pub best_features: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfiguration {
    pub sublinear_tf: bool,
    pub word_clusters_name: Option<String>,
    pub use_stemming: bool,
}

#[derive(Debug, Deserialize)]
pub struct TfIdfVectorizerModel {
    pub idf_diag: Vec<f32>,
    pub vocab: HashMap<String, usize>,
}
