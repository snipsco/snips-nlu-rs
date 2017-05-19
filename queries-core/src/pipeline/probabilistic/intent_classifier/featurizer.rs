#[macro_use(s)]
use std::collections::{HashMap, HashSet};

use ndarray::prelude::*;

use errors::*;
use preprocessing::light::tokenize;

pub struct Featurizer {
    best_features: Array1<usize>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Array2<f32>,
    stop_words: HashSet<String>
}

impl Featurizer {
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let words: Vec<String> = tokenize(input).into_iter().filter_map(|t|
            if !self.stop_words.contains(&t.value) {
                Some(t.value)
            } else {
                None
            }
        ).collect();
        let vocabulary_size = self.vocabulary.values().max().unwrap() + 1;
        let mut words_count: Array2<f32> = Array::zeros((1, vocabulary_size));
        for word in words {
            if let Some(word_idx) = self.vocabulary.get(&word) {
                words_count[[0, *word_idx]] += 1.;
            }
        }
        let mut weighted_words_count = words_count.dot(&self.idf_diag).subview(Axis(0), 0).to_owned();
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
        let best_features = array![0, 1, 3, 5];
        let vocabulary = hashmap![
            "beautiful".to_string() => 0,
            "bird".to_string() => 1,
            "hello".to_string() => 2,
            "here".to_string() => 3,
            "you".to_string() => 4,
            "world".to_string() => 5,
        ];

        let stop_words = hashset!["this".to_string(), "is".to_string()];

        let idf_diag = array![[0.34, 0., 0., 0., 0., 0.],
                              [0., 0.97, 0., 0., 0., 0.],
                              [0., 0., 1.21, 0., 0., 0.],
                              [0., 0., 0., 1.07, 0., 0.],
                              [0., 0., 0., 0., 0.54, 0.],
                              [0., 0., 0., 0., 0., 0.88]];

        let features_processor = Featurizer {
            best_features,
            vocabulary,
            idf_diag,
            stop_words
        };

        // When
        let features = features_processor.transform("hello this bird is a beautiful bird").unwrap();

        // Then
        let expected_features = array![0.1470869484, 0.8392608234, 0., 0.];
        assert_epsilon_eq_array1(&features, &expected_features, 1e-6);
    }
}