use std::collections::HashMap;
use itertools::Itertools;

use ndarray::prelude::*;

use errors::*;
use nlu_utils::token::tokenize_light;
use pipeline::probabilistic::configuration::FeaturizerConfiguration;
use nlu_utils::string::normalize;
use models::stemmer::{Stemmer, StaticMapStemmer};
use nlu_utils::token::compute_all_ngrams;

pub struct Featurizer {
    language_code: String,
    best_features: Array1<usize>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Vec<f32>,
    entity_utterances_to_feature_names: HashMap<String, Vec<String>>
}

impl Featurizer {
    pub fn new(config: FeaturizerConfiguration) -> Self {
        let language_code = config.language_code;
        let best_features = Array::from_iter(config.best_features);
        let vocabulary = config.tfidf_vectorizer_vocab;
        let idf_diag = config.tfidf_vectorizer_idf_diag;
        let entity_utterances_to_feature_names = config.entity_utterances_to_feature_names;

        Self { language_code, best_features, vocabulary, idf_diag, entity_utterances_to_feature_names }
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let preprocessed_input = self.preprocess_query(input);
        let words = tokenize_light(&preprocessed_input);

        let vocabulary_size = self.vocabulary.values().max().unwrap() + 1;
        let mut words_count = vec![0.; vocabulary_size];
        for word in words {
            if let Some(word_idx) = self.vocabulary.get(&word) {
                words_count[*word_idx] += self.idf_diag[*word_idx];
            }
        }

        let l2_norm: f32 = words_count.iter().fold(0., |norm, v| norm + v * v).sqrt();
        let safe_l2_norm = if l2_norm > 0. { l2_norm } else { 1. };
        words_count = words_count.iter().map(|c| *c / safe_l2_norm).collect_vec();
        let selected_features = Array::from_iter(
            (0..self.best_features.len()).map(|fi| words_count[self.best_features[fi]])
        );
        Ok(selected_features)
    }

    fn preprocess_query(&self, query: &str) -> String {
        let normalized_query = normalize(query);
        let normalized_stemmed_query = StaticMapStemmer::new(self.language_code.clone()).ok()
            .map(|stemmer| stem_sentence(&normalized_query, &stemmer))
            .unwrap_or(normalized_query);
        self.add_entity_to_query(&normalized_stemmed_query)
    }

    fn add_entity_to_query(&self, query: &str) -> String {
        let tokens = tokenize_light(query);
        let tokens_ref = tokens.iter().map(|t| &**t).collect_vec();
        let ngrams = compute_all_ngrams(&*tokens_ref, tokens_ref.len());

        let matching_features: Vec<&String> = ngrams.iter()
            .filter_map(|ngrams| self.entity_utterances_to_feature_names.get(&ngrams.0))
            .flat_map(|features| features)
            .collect();

        if matching_features.len() > 0 {
            format!("{} {}", query, matching_features.iter().join(" "))
        } else {
            query.to_string()
        }
    }
}


fn stem_sentence<S: Stemmer>(input: &str, stemmer: &S) -> String {
    let stemmed_words: Vec<_> = tokenize_light(input)
        .iter()
        .map(|word| stemmer.stem(word))
        .collect();
    stemmed_words.join(" ")
}


#[cfg(test)]
mod tests {
    use super::Featurizer;
    use testutils::assert_epsilon_eq_array1;

    #[test]
    fn preprocess_query_works() {
        // Given
        let language_code = String::from("en");
        let best_features = array![];
        let vocabulary = hashmap![];
        let idf_diag = vec![];
        let entity_utterances_to_feature_names = hashmap![
            "bird".to_string() => vec!["featureentityanimal".to_string()],
            "hello".to_string() => vec!["featureentityword".to_string(), "featureentitygreeting".to_string()]
        ];

        let featurizer = Featurizer {
            language_code,
            best_features,
            vocabulary,
            idf_diag,
            entity_utterances_to_feature_names
        };

        let input = "héLLo thIs bïrd is a beaUtiful bird";

        // When
        let preprocessed_input = featurizer.preprocess_query(input);
        let expected_preprocessd_input = "hello this bird be a beauti bird featureentityword featureentitygreeting featureentityanimal featureentityanimal";

        // Then
        assert_eq!(preprocessed_input, expected_preprocessd_input)
    }

    #[test]
    fn transform_works() {
        // Given
        let language_code = String::from("en");
        let best_features = array![0, 1, 2, 3, 6, 7, 8, 9];
        let vocabulary = hashmap![
            "awful".to_string() => 0,
            "beauti".to_string() => 1,
            "bird".to_string() => 2,
            "blue".to_string() => 3,
            "hello".to_string() => 4,
            "nice".to_string() => 5,
            "world".to_string() => 6,
            "featureentityanimal".to_string() => 7,
            "featureentityword".to_string() => 8,
            "featureentitygreeting".to_string() => 9
        ];

        let idf_diag = vec![2.252762968495368,
                            2.252762968495368,
                            1.5596157879354227,
                            2.252762968495368,
                            1.8472978603872037,
                            1.8472978603872037,
                            1.5596157879354227,
                            0.7,
                            1.7,
                            2.7
        ];

        let entity_utterances_to_feature_names = hashmap![
            "bird".to_string() => vec!["featureentityanimal".to_string()],
            "hello".to_string() => vec!["featureentityword".to_string(), "featureentitygreeting".to_string()]
        ];

        let featurizer = Featurizer {
            language_code,
            best_features,
            vocabulary,
            idf_diag,
            entity_utterances_to_feature_names
        };

        // When
        let features = featurizer.transform("hello this bird is a beautiful bird").unwrap();

        // Then
        let expected_features = array![0.0, 0.40887040136658365, 0.5661321160803057, 0.0, 0.0, 0.2540962231350679, 0.30854541380686823, 0.4900427160462025];
        assert_epsilon_eq_array1(&features, &expected_features, 1e-6);
    }
}
