use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;
use ndarray::prelude::*;
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::string::normalize;
use nlu_utils::token::{compute_all_ngrams, tokenize_light};
use snips_nlu_ontology::{BuiltinEntityKind, Language};

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::FeaturizerModel;
use crate::resources::stemmer::Stemmer;
use crate::resources::word_clusterer::WordClusterer;
use crate::resources::SharedResources;

pub struct Featurizer {
    best_features: Vec<usize>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Vec<f32>,
    sublinear: bool,
    word_clusterer: Option<Arc<WordClusterer>>,
    stemmer: Option<Arc<Stemmer>>,
    shared_resources: Arc<SharedResources>,
    language: Language,
}

impl Featurizer {
    pub fn new(model: FeaturizerModel, shared_resources: Arc<SharedResources>) -> Result<Self> {
        let best_features = model.best_features;
        let vocabulary = model.tfidf_vectorizer.vocab;
        let language = Language::from_str(model.language_code.as_ref())?;
        let idf_diag = model.tfidf_vectorizer.idf_diag;
        let opt_word_clusterer = if let Some(clusters_name) = model.config.word_clusters_name {
            Some(
                shared_resources
                    .word_clusterers
                    .get(&clusters_name)
                    .cloned()
                    .ok_or_else(|| {
                        format_err!(
                            "Cannot find word clusters '{}' in shared resources",
                            clusters_name
                        )
                    })?,
            )
        } else {
            None
        };
        let stemmer = if model.config.use_stemming {
            Some(
                shared_resources
                    .stemmer
                    .as_ref()
                    .cloned()
                    .ok_or_else(|| format_err!("Cannot find stemmer in shared resources"))?,
            )
        } else {
            None
        };

        Ok(Self {
            best_features,
            vocabulary,
            idf_diag,
            sublinear: model.config.sublinear_tf,
            word_clusterer: opt_word_clusterer,
            stemmer,
            shared_resources,
            language,
        })
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let preprocessed_tokens = self.preprocess_utterance(input)?;
        let vocabulary_size = self.vocabulary.values().max().unwrap() + 1;

        let mut tfidf: Vec<f32> = vec![0.; vocabulary_size];
        let mut match_idx: HashSet<usize> = HashSet::new();
        for word in preprocessed_tokens {
            if let Some(word_idx) = self.vocabulary.get(&word) {
                tfidf[*word_idx] += 1.;
                match_idx.insert(*word_idx);
            }
        }

        for ix in match_idx {
            if self.sublinear {
                tfidf[ix] = (tfidf[ix].ln() + 1.) * self.idf_diag[ix]
            } else {
                tfidf[ix] *= self.idf_diag[ix]
            }
        }

        let l2_norm: f32 = tfidf.iter().fold(0., |norm, v| norm + v * v).sqrt();
        let safe_l2_norm = if l2_norm > 0. { l2_norm } else { 1. };

        tfidf = tfidf.iter().map(|c| *c / safe_l2_norm).collect_vec();

        let selected_features =
            Array::from_iter((0..self.best_features.len()).map(|fi| tfidf[self.best_features[fi]]));
        Ok(selected_features)
    }

    fn preprocess_utterance(&self, utterance: &str) -> Result<Vec<String>> {
        let language = NluUtilsLanguage::from_language(self.language);
        let tokens = tokenize_light(utterance, language);
        let word_cluster_features = self
            .word_clusterer
            .clone()
            .map(|clusterer| get_word_cluster_features(&tokens, clusterer))
            .unwrap_or_else(|| vec![]);
        let normalized_stemmed_tokens = normalize_stem(&tokens, self.stemmer.clone());
        let normalized_stemmed_string = normalized_stemmed_tokens.join(" ");
        let custom_entities_features: Vec<String> = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(&normalized_stemmed_string, None)?
            .into_iter()
            .map(|entity| get_custom_entity_feature_name(&*entity.entity_identifier, language))
            .collect();

        let builtin_entities = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(utterance, None, true)?;
        let builtin_entities_features: Vec<String> = builtin_entities
            .iter()
            .map(|ent| get_builtin_entity_feature_name(ent.entity_kind, language))
            .sorted();

        Ok(vec![
            normalized_stemmed_tokens,
            builtin_entities_features,
            custom_entities_features,
            word_cluster_features,
        ]
        .into_iter()
        .flat_map(|features| features)
        .collect())
    }
}

fn get_builtin_entity_feature_name(
    entity_kind: BuiltinEntityKind,
    language: NluUtilsLanguage,
) -> String {
    let e = tokenize_light(&entity_kind.identifier().to_lowercase(), language).join("");
    format!("builtinentityfeature{}", e)
}

fn get_custom_entity_feature_name(entity_name: &str, language: NluUtilsLanguage) -> String {
    let e = tokenize_light(&entity_name.to_lowercase(), language).join("");
    format!("entityfeature{}", e)
}

fn get_word_cluster_features(
    query_tokens: &[String],
    word_clusterer: Arc<WordClusterer>,
) -> Vec<String> {
    let tokens_ref = query_tokens.iter().map(|t| t.as_ref()).collect_vec();
    compute_all_ngrams(tokens_ref.as_ref(), tokens_ref.len())
        .into_iter()
        .filter_map(|ngram| word_clusterer.get_cluster(&ngram.0.to_lowercase()))
        .sorted()
}

fn normalize_stem(tokens: &[String], opt_stemmer: Option<Arc<Stemmer>>) -> Vec<String> {
    opt_stemmer
        .map(|stemmer| tokens.iter().map(|t| stemmer.stem(&normalize(t))).collect())
        .unwrap_or_else(|| tokens.iter().map(|t| normalize(t)).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::sync::Arc;

    use nlu_utils::language::Language;
    use nlu_utils::token::tokenize_light;

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::models::{FeaturizerConfiguration, FeaturizerModel, TfIdfVectorizerModel};
    use crate::resources::stemmer::HashMapStemmer;
    use crate::resources::word_clusterer::HashMapWordClusterer;
    use crate::resources::SharedResources;
    use crate::testutils::assert_epsilon_eq_array1;
    use crate::testutils::MockedBuiltinEntityParser;
    use crate::testutils::MockedCustomEntityParser;

    use super::{get_word_cluster_features, Featurizer};

    #[test]
    fn transform_works() {
        // Given
        let mocked_custom_parser = MockedCustomEntityParser::from_iter(vec![(
            "hello this bird is a beauti bird".to_string(),
            vec![
                CustomEntity {
                    value: "hello".to_string(),
                    resolved_value: "hello".to_string(),
                    range: 0..5,
                    entity_identifier: "greeting".to_string(),
                },
                CustomEntity {
                    value: "hello".to_string(),
                    resolved_value: "hello".to_string(),
                    range: 0..5,
                    entity_identifier: "word".to_string(),
                },
                CustomEntity {
                    value: "bird".to_string(),
                    resolved_value: "bird".to_string(),
                    range: 11..15,
                    entity_identifier: "animal".to_string(),
                },
                CustomEntity {
                    value: "bird".to_string(),
                    resolved_value: "bird".to_string(),
                    range: 31..35,
                    entity_identifier: "animal".to_string(),
                },
            ],
        )]);
        let mocked_builtin_parser = MockedBuiltinEntityParser {
            mocked_outputs: HashMap::new(),
        };
        let mocked_stemmer =
            HashMapStemmer::from_iter(vec![("beautiful".to_string(), "beauti".to_string())]);

        let resources = SharedResources {
            custom_entity_parser: Arc::new(mocked_custom_parser),
            builtin_entity_parser: Arc::new(mocked_builtin_parser),
            stemmer: Some(Arc::new(mocked_stemmer)),
            word_clusterers: HashMap::new(),
            gazetteers: HashMap::new(),
            stop_words: HashSet::new(),
        };
        let best_features = vec![0, 1, 2, 3, 6, 7, 8, 9];
        let vocab = hashmap![
            "awful".to_string() => 0,
            "beauti".to_string() => 1,
            "bird".to_string() => 2,
            "blue".to_string() => 3,
            "hello".to_string() => 4,
            "nice".to_string() => 5,
            "world".to_string() => 6,
            "entityfeatureanimal".to_string() => 7,
            "entityfeatureword".to_string() => 8,
            "entityfeaturegreeting".to_string() => 9
        ];

        let idf_diag = vec![
            2.252762968495368,
            2.252762968495368,
            1.5596157879354227,
            2.252762968495368,
            1.8472978603872037,
            1.8472978603872037,
            1.5596157879354227,
            0.7,
            1.7,
            2.7,
        ];

        let language_code = "en";
        let tfidf_vectorizer = TfIdfVectorizerModel { idf_diag, vocab };

        let featurizer_config = FeaturizerModel {
            language_code: language_code.to_string(),
            tfidf_vectorizer,
            config: FeaturizerConfiguration {
                sublinear_tf: false,
                word_clusters_name: None,
                use_stemming: true,
            },
            best_features,
        };

        let featurizer = Featurizer::new(featurizer_config, Arc::new(resources)).unwrap();

        // When
        let input = "Hëllo this bïrd is a beautiful Bïrd";
        let features = featurizer.transform(input).unwrap();

        // Then
        let expected_features = array![
            0.0,
            0.40887040136658365,
            0.5661321160803057,
            0.0,
            0.0,
            0.2540962231350679,
            0.30854541380686823,
            0.4900427160462025
        ];
        assert_epsilon_eq_array1(&expected_features, &features, 1e-6);
    }

    #[test]
    fn get_word_cluster_features_works() {
        // Given
        let language = Language::EN;
        let query_tokens = tokenize_light("I, love House, muSic", language);
        let word_clusterer = HashMapWordClusterer::from_iter(vec![
            ("love".to_string(), "cluster_love".to_string()),
            ("house".to_string(), "cluster_house".to_string()),
        ]);

        // When
        let augmented_query = get_word_cluster_features(&query_tokens, Arc::new(word_clusterer));

        // Then
        let expected_augmented_query =
            vec!["cluster_house".to_string(), "cluster_love".to_string()];
        assert_eq!(augmented_query, expected_augmented_query)
    }
}
