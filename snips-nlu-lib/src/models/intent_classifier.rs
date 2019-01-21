use std::collections::HashMap;

use crate::utils::IntentName;

#[derive(Debug, Deserialize)]
pub struct IntentClassifierModel {
    pub featurizer: Option<String>,
    pub intercept: Option<Vec<f32>>,
    pub coeffs: Option<Vec<Vec<f32>>>,
    pub intent_list: Vec<Option<IntentName>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerModel {
    pub language_code: String,
    pub tfidf_vectorizer: String,
    pub cooccurrence_vectorizer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TfidfVectorizerModel {
    pub language_code: String,
    pub builtin_entity_scope: Vec<String>,
    pub vectorizer: SklearnVectorizerModel,
    pub config: TfidfVectorizerConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct TfidfVectorizerConfiguration {
    pub use_stemming: bool,
    pub word_clusters_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SklearnVectorizerModel {
    pub idf_diag: Vec<f32>,
    pub vocab: HashMap<String, usize>,
}

#[derive(Debug, Deserialize)]
pub struct CooccurrenceVectorizerModel {
    pub language_code: String,
    pub builtin_entity_scope: Vec<String>,
    pub word_pairs: HashMap<usize, (String, String)>,
    pub config: CooccurrenceVectorizerConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct CooccurrenceVectorizerConfiguration {
    pub window_size: Option<usize>,
    pub filter_stop_words: bool,
}
