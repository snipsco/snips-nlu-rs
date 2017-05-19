use std::collections::HashMap;

use serde_json;

//#[derive(Debug, Deserialize)]
//pub enum TaggingScheme {
    //IO,
    //BIO,
    //BILOU,
//}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticParserConfiguration {
    language_code: String,
    taggers: HashMap<String, TaggerConfiguration>,
    intent_classifier: ProbabilisticIntentClassifierConfiguration,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ProbabilisticIntentClassifierConfiguration {
    featurizer: Option<FeaturizerConfiguration>,
    intercept: Option<Vec<f32>>,
    coeffs: Option<Vec<Vec<f32>>>,
    intent_list: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturizerConfiguration {
    language_code: String,
    tfidf_vectorizer_idf_diag: Vec<f32>,
    pvalue_threshold: f32,
    best_features: Vec<usize>,
    tfidf_vectorizer_vocab: HashMap<String, usize>,
    tfidf_vectorizer_stop_words: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TaggerConfiguration {
    //language: String, // cause a duplicate error ?
    crf_model_data: String,
    tagging_scheme: u8, // should be a enum but deserializing an int to enum seems not trivial
    features_signatures: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    factory_name: String,
    offsets: Vec<i32>,
    args: HashMap<String, serde_json::Value>,
}
