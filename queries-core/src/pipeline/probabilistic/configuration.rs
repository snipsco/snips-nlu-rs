use std::collections::HashMap;

use serde_json;

//#[derive(Debug, Deserialize)]
//pub enum TaggingScheme {
    //pub IO,
    //pub BIO,
    //pub BILOU,
//}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserConfiguration {
    pub language_code: String,
    pub taggers: HashMap<String, TaggerConfiguration>,
    pub intent_classifier: ProbabilisticIntentClassifierConfiguration,
    pub slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticIntentClassifierConfiguration {
    pub featurizer: Option<FeaturizerConfiguration>,
    pub intercept: Option<Vec<f32>>,
    pub coeffs: Option<Vec<Vec<f32>>>,
    pub intent_list: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfiguration {
    pub language_code: String,
    pub tfidf_vectorizer_idf_diag: Vec<f32>,
    pub pvalue_threshold: f32,
    pub best_features: Vec<usize>,
    pub tfidf_vectorizer_vocab: HashMap<String, usize>,
    pub tfidf_vectorizer_stop_words: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TaggerConfiguration {
    //language: String, // cause a duplicate error ?
    pub crf_model_data: String,
    pub tagging_scheme: u8, // should be a enum but deserializing an int to enum seems not trivial
    pub features_signatures: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    pub factory_name: String,
    pub offsets: Vec<i32>,
    pub args: HashMap<String, serde_json::Value>,
}
