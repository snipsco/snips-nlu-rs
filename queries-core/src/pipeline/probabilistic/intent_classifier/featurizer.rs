use std::collections::{HashMap, HashSet};

use ndarray::prelude::*;

use errors::*;
use preprocessing::tokenize;
use pipeline::probabilistic::configuration::FeaturizerConfiguration;
use std::iter::FromIterator;

pub struct Featurizer {
    best_features: Array1<usize>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Array2<f32>,
    stop_words: HashSet<String>
}

impl Featurizer {
    pub fn new(config: FeaturizerConfiguration) -> Self {
        let best_features = Array::from_iter(config.best_features);
        let vocabulary = config.tfidf_vectorizer_vocab;
        let dimension = config.tfidf_vectorizer_idf_diag.len();
        let mut idf_diag: Array2<f32> = Array::zeros((dimension, dimension));
        for (i, diag_el) in config.tfidf_vectorizer_idf_diag.into_iter().enumerate() {
            idf_diag[[i, i]] = diag_el;
        }

        let stop_words = config.tfidf_vectorizer_stop_words
            .map(HashSet::from_iter)
            .unwrap_or(HashSet::new());

        Self {
            best_features,
            vocabulary,
            idf_diag,
            stop_words
        }
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let ref normalized_input = input.to_lowercase();
        let words = tokenize(normalized_input).into_iter().filter_map(|t|
            if !self.stop_words.contains(&t.value) {
                Some(t.value)
            } else {
                None
            }
        );
        let vocabulary_size = self.vocabulary.values().max().unwrap() + 1;
        let mut words_count: Array2<f32> = Array::zeros((1, vocabulary_size));
        for word in words {
            if let Some(word_idx) = self.vocabulary.get(&word) {
                words_count[[0, *word_idx]] += 1.;
            }
        }
        let mut weighted_words_count = words_count.dot(&self.idf_diag)
            .subview(Axis(0), 0)
            .to_owned();
        let l2_norm: f32 = weighted_words_count.iter().fold(0., |norm, v| norm + v * v).sqrt();
        weighted_words_count = weighted_words_count
            .map(|c|
                if l2_norm > 0. {
                    *c / l2_norm
                } else {
                    *c
                }
            );
        let selected_features = Array::from_iter(
            (0..self.best_features.len()).map(|fi| weighted_words_count[self.best_features[fi]])
        );
        return Ok(selected_features);
    }
}

#[cfg(test)]
mod tests {
    use super::Featurizer;
    use testutils::assert_epsilon_eq_array1;

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

        let stop_words = hashset!["the".to_string(), "is".to_string()];

        let idf_diag = array![[2.252762968495368, 0., 0., 0., 0., 0., 0.],
                              [0., 2.252762968495368, 0., 0., 0., 0., 0.],
                              [0., 0., 1.5596157879354227, 0., 0., 0., 0.],
                              [0., 0., 0., 2.252762968495368, 0., 0., 0.],
                              [0., 0., 0., 0., 1.8472978603872037, 0., 0.],
                              [0., 0., 0., 0., 0., 1.8472978603872037, 0.],
                              [0., 0., 0., 0., 0., 0., 1.5596157879354227]];

        let featurizer = Featurizer {
            best_features,
            vocabulary,
            idf_diag,
            stop_words
        };

        // When
        let features = featurizer.transform("hello this bird is a beautiful bird").unwrap();

        // Then
        let expected_features = array![0., 0.527808526514, 0.730816799167, 0., 0.];
        assert_epsilon_eq_array1(&features, &expected_features, 1e-6);
    }
}
