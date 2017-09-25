use std::collections::HashMap;
use itertools::Itertools;

use ndarray::prelude::*;

use errors::*;
use models::word_clusterer::{StaticMapWordClusterer, WordClusterer};
use nlu_utils::token::compute_all_ngrams;
use language::LanguageConfig;
use nlu_utils::language::Language;
use nlu_utils::token::tokenize_light;
use pipeline::probabilistic::configuration::FeaturizerConfiguration;
use std::str::FromStr;

pub struct Featurizer {
    best_features: Array1<usize>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Vec<f32>,
    language_config: LanguageConfig,
    word_clusterer: Option<StaticMapWordClusterer>,
}

impl Featurizer {
    pub fn new(config: FeaturizerConfiguration) -> Self {
        let best_features = Array::from_iter(config.best_features);
        let vocabulary = config.tfidf_vectorizer.vocab;
        let idf_diag = config.tfidf_vectorizer.idf_diag;
        let language_config = LanguageConfig::from_str(&config.language_code).unwrap();
        let word_clusterer = match language_config.intent_classification_clusters() {
            Some(clusters_name) => StaticMapWordClusterer::new(language_config.language, clusters_name.to_string()).ok(),
            None => None
        };

        Self { best_features, vocabulary, idf_diag, language_config, word_clusterer }
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let preprocessed_query = preprocess_query(input, self.language_config.language, &self.word_clusterer);

        let words = tokenize_light(&preprocessed_query, self.language_config.language);

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
}

fn add_word_cluster_features_to_query<C: WordClusterer>(query: &str, language: Language, word_clusterer: &C) -> String {
    let tokens = tokenize_light(query, language);
    let tokens_ref: Vec<&str> = tokens
        .iter()
        .map(|string| &**string)
        .collect_vec();
    let query_clusters: Vec<String> = compute_all_ngrams(&tokens_ref[..], tokens_ref.len())
        .into_iter()
        .filter_map(|ngram| word_clusterer.get_cluster(&ngram.0))
        .collect();

    if query_clusters.len() > 0 {
        format!("{} {}", query, query_clusters.join(" "))
    } else {
        query.to_string()
    }
}

fn preprocess_query<C: WordClusterer>(query: &str, language: Language, word_clusterer: &Option<C>) -> String {
    if let Some(ref clusterer) = *word_clusterer {
        add_word_cluster_features_to_query(query, language, clusterer)
    } else {
        query.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Featurizer, add_word_cluster_features_to_query};
    use std::str::FromStr;

    use testutils::assert_epsilon_eq_array1;
    use language::LanguageConfig;
    use models::word_clusterer::{StaticMapWordClusterer, WordClusterer};
    use nlu_utils::language::Language;
    use nlu_utils::string::normalize;

    #[test]
    fn transform_works() {
        // Given
        let best_features = array![0, 1, 2, 3, 6];
        let vocabulary = hashmap![
            "awful".to_string() => 0,
            "beautiful".to_string() => 1,
            "bird".to_string() => 2,
            "blue".to_string() => 3,
            "hello".to_string() => 4,
            "nice".to_string() => 5,
            "world".to_string() => 6
        ];

        let idf_diag = vec![2.252762968495368,
                            2.252762968495368,
                            1.5596157879354227,
                            2.252762968495368,
                            1.8472978603872037,
                            1.8472978603872037,
                            1.5596157879354227];
        let language_code = "en";
        let language = Language::from_str(language_code).unwrap();
        let language_config = LanguageConfig::from_str(&language_code).unwrap();
        let word_clusterer = StaticMapWordClusterer::new(language, "brown_cluster".to_string()).ok();

        let featurizer = Featurizer {
            best_features,
            vocabulary,
            idf_diag,
            language_config,
            word_clusterer,
        };

        // When
        let features = featurizer.transform("hello this bird is a beautiful bird").unwrap();

        // Then
        let expected_features = array![0., 0.527808526514, 0.730816799167, 0., 0.];
        assert_epsilon_eq_array1(&features, &expected_features, 1e-6);
    }

    struct TestWordClusterer {}

    impl WordClusterer for TestWordClusterer {
        fn get_cluster(&self, word: &str) -> Option<String> {
            match &*normalize(word) {
                "love" => Some("cluster_love".to_string()),
                "house" => Some("cluster_house".to_string()),
                _ => None
            }
        }
    }

    #[test]
    fn add_word_cluster_features_to_query_works() {
        // Given
        let language = Language::EN;
        let query = "I, love Höuse, Müsic";
        let word_clusterer = TestWordClusterer {};

        // When
        let augmented_query = &add_word_cluster_features_to_query(query, language, &word_clusterer);

        // Then
        let expected_augmented_query = "I, love Höuse, Müsic cluster_love cluster_house";
        assert_eq!(augmented_query, expected_augmented_query)
    }
}
