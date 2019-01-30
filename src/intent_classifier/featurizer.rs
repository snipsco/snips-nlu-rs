use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::{format_err, ResultExt};
use itertools::Itertools;
use ndarray::prelude::*;
use snips_nlu_ontology::{BuiltinEntityKind, Language};
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_utils::string::normalize;
use snips_nlu_utils::token::{compute_all_ngrams, tokenize_light};

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::{CooccurrenceVectorizerModel, FeaturizerModel, TfidfVectorizerModel};
use crate::resources::stemmer::Stemmer;
use crate::resources::word_clusterer::WordClusterer;
use crate::resources::SharedResources;
use crate::utils::{replace_entities, MatchedEntity};

type WordPair = (String, String);

pub struct Featurizer {
    tfidf_vectorizer: TfidfVectorizer,
    cooccurrence_vectorizer: Option<CooccurrenceVectorizer>,
}

impl Featurizer {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let featurizer_model_path = path.as_ref().join("featurizer.json");
        let model_file = File::open(&featurizer_model_path).with_context(|_| {
            format!("Cannot open Featurizer file '{:?}'", &featurizer_model_path)
        })?;
        let model: FeaturizerModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize FeaturizerModel json data")?;

        // Load tf-idf vectorizer
        let tfidf_vectorizer_path = path.as_ref().join(model.tfidf_vectorizer);
        let tfidf_vectorizer =
            TfidfVectorizer::from_path(&tfidf_vectorizer_path, shared_resources.clone())?;

        // Load cooccurrence vectorizer
        let cooccurrence_vectorizer: Result<Option<CooccurrenceVectorizer>> =
            if let Some(cooccurrence_name) = model.cooccurrence_vectorizer {
                let cooccurrence_vectorizer_path = path.as_ref().join(cooccurrence_name);
                let vectorizer = CooccurrenceVectorizer::from_path(
                    &cooccurrence_vectorizer_path,
                    shared_resources.clone(),
                )?;
                Ok(Some(vectorizer))
            } else {
                Ok(None)
            };

        Ok(Self {
            tfidf_vectorizer,
            cooccurrence_vectorizer: cooccurrence_vectorizer?,
        })
    }
}

impl Featurizer {
    #[cfg(test)]
    pub fn new(
        tfidf_vectorizer: TfidfVectorizer,
        cooccurrence_vectorizer: Option<CooccurrenceVectorizer>,
    ) -> Self {
        Self {
            tfidf_vectorizer,
            cooccurrence_vectorizer,
        }
    }

    pub fn transform(&self, input: &str) -> Result<Array1<f32>> {
        let mut features = self.tfidf_vectorizer.transform(input)?;
        if let Some(vectorizer) = self.cooccurrence_vectorizer.as_ref() {
            let cooccurrence_features: Vec<f32> = vectorizer.transform(input)?;
            features.extend(cooccurrence_features)
        };
        Ok(Array::from_iter(features))
    }
}

pub struct TfidfVectorizer {
    builtin_entity_scope: Vec<BuiltinEntityKind>,
    vocabulary: HashMap<String, usize>,
    idf_diag: Vec<f32>,
    word_clusterer: Option<Arc<WordClusterer>>,
    stemmer: Option<Arc<Stemmer>>,
    language: NluUtilsLanguage,
    shared_resources: Arc<SharedResources>,
}

impl TfidfVectorizer {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let parser_model_path = path.as_ref().join("vectorizer.json");
        let model_file = File::open(&parser_model_path).with_context(|_| {
            format!(
                "Cannot open TfidfVectorizer file '{:?}'",
                &parser_model_path
            )
        })?;
        let model: TfidfVectorizerModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize TfidfVectorizer json data")?;
        Self::new(model, shared_resources)
    }
}

impl TfidfVectorizer {
    pub fn new(
        model: TfidfVectorizerModel,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let vocabulary = model.vectorizer.vocab;
        let idf_diag = model.vectorizer.idf_diag;

        let ontology_language = Language::from_str(model.language_code.as_ref())?;
        let language = NluUtilsLanguage::from_language(ontology_language);

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

        let builtin_entity_scope = model
            .builtin_entity_scope
            .iter()
            .map(|ent| {
                BuiltinEntityKind::from_identifier(ent)
                    .map_err(|_| format_err!("Unknown builtin entity {:?}", ent))
            })
            .collect::<Result<Vec<BuiltinEntityKind>>>()?;

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
            builtin_entity_scope,
            vocabulary,
            idf_diag,
            word_clusterer: opt_word_clusterer,
            stemmer,
            language,
            shared_resources,
        })
    }

    pub fn transform(&self, utterance: &str) -> Result<Vec<f32>> {
        let tokens = tokenize_light(utterance, self.language);
        let normalized_tokens = normalize_stem(&tokens, self.stemmer.clone());

        // Extract builtin entities on the raw utterance
        let builtin_entities = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(utterance, Some(&self.builtin_entity_scope[..]), true)?;

        let builtin_entities_features: Vec<String> = builtin_entities
            .iter()
            .map(|ent| get_builtin_entity_feature_name(ent.entity_kind, self.language))
            .sorted();

        // Extract custom entities on the normalized utterance
        let custom_entities = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(&*normalized_tokens.join(" "), None)?;

        let custom_entities_features: Vec<String> = custom_entities
            .into_iter()
            .map(|ent| get_custom_entity_feature_name(&*ent.entity_identifier, self.language))
            .collect();

        // Extract word clusters on the raw utterance
        let word_clusters = self
            .word_clusterer
            .clone()
            .map(|clusterer| get_word_clusters(&tokens, clusterer))
            .unwrap_or_else(|| vec![]);

        // Compute tf-idf features
        let features_it = &[
            normalized_tokens,
            builtin_entities_features,
            custom_entities_features,
            word_clusters,
        ];

        let vocabulary_size = self.vocabulary.values().max().unwrap() + 1;
        let mut features: Vec<f32> = vec![0.; vocabulary_size];
        let mut match_idx: HashSet<usize> = HashSet::new();
        for extracted_features in features_it.iter() {
            for word in extracted_features {
                if let Some(word_idx) = self.vocabulary.get(word) {
                    features[*word_idx] += 1.;
                    match_idx.insert(*word_idx);
                }
            }
        }

        for ix in match_idx {
            features[ix] *= self.idf_diag[ix]
        }

        // Normalize tf-idf
        let l2_norm: f32 = features.iter().fold(0., |norm, v| norm + v * v).sqrt();
        let safe_l2_norm = if l2_norm > 0. { l2_norm } else { 1. };
        features = features.iter().map(|c| *c / safe_l2_norm).collect_vec();
        Ok(features)
    }
}

pub struct CooccurrenceVectorizer {
    language: NluUtilsLanguage,
    builtin_entity_scope: Vec<BuiltinEntityKind>,
    word_pairs: HashMap<WordPair, usize>,
    filter_stop_words: bool,
    window_size: Option<usize>,
    keep_order: bool,
    unknown_words_replacement_string: Option<String>,
    shared_resources: Arc<SharedResources>,
}

impl CooccurrenceVectorizer {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let parser_model_path = path.as_ref().join("vectorizer.json");
        let model_file = File::open(&parser_model_path).with_context(|_| {
            format!(
                "Cannot open CooccurrenceVectorizer file '{:?}'",
                &parser_model_path
            )
        })?;
        let model: CooccurrenceVectorizerModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize CooccurrenceVectorizer json data")?;
        Self::new(model, shared_resources)
    }
}

impl CooccurrenceVectorizer {
    pub fn new(
        model: CooccurrenceVectorizerModel,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let builtin_entity_scope = model
            .builtin_entity_scope
            .iter()
            .map(|ent| {
                BuiltinEntityKind::from_identifier(ent)
                    .map_err(|_| format_err!("Unknown builtin entity {:?}", ent))
            })
            .collect::<Result<Vec<BuiltinEntityKind>>>()?;

        let word_pairs = model
            .word_pairs
            .into_iter()
            .map(|(index, pairs)| (pairs, index))
            .collect();

        let filter_stop_words = model.config.filter_stop_words;
        let window_size = model.config.window_size;
        let keep_order = model.config.keep_order;
        let unknown_words_replacement_string = model.config.unknown_words_replacement_string;

        let ontology_language = Language::from_str(model.language_code.as_ref())?;
        let language = NluUtilsLanguage::from_language(ontology_language);

        Ok(Self {
            language,
            builtin_entity_scope,
            word_pairs,
            filter_stop_words,
            window_size,
            keep_order,
            unknown_words_replacement_string,
            shared_resources,
        })
    }

    fn transform(&self, utterance: &str) -> Result<Vec<f32>> {
        // Extract builtin entities on the raw utterance
        let builtin_entities = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(utterance, Some(&self.builtin_entity_scope[..]), true)?;

        // Extract custom entities on the raw utterance
        let custom_entities = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(utterance, None)?;

        let matched_builtins = builtin_entities.into_iter().map(|entity| entity.into());

        let matched_customs = custom_entities.into_iter().map(|entity| entity.into());

        let mut matched_entities: Vec<MatchedEntity> = vec![];
        matched_entities.extend(matched_builtins);
        matched_entities.extend(matched_customs);

        let enriched_utterance = replace_entities(utterance, matched_entities, |ent_kind| {
            self.placeholder_fn(ent_kind)
        })
        .1;

        let tokens = tokenize_light(&*enriched_utterance, self.language);

        let mut features: Vec<f32> = vec![0.; self.word_pairs.len()];
        for pair in self.extract_word_pairs(tokens) {
            self.word_pairs.get(&pair).map(|pair_index| {
                features[*pair_index] = 1.0;
            });
        }
        Ok(features)
    }

    fn placeholder_fn(&self, entity_kind: &str) -> String {
        tokenize_light(entity_kind, self.language)
            .join("")
            .to_uppercase()
    }

    fn extract_word_pairs(&self, tokens: Vec<String>) -> HashSet<WordPair> {
        let filtered_tokens: Vec<String> = tokens
            .into_iter()
            .filter(|t| {
                !(self.filter_stop_words && self.shared_resources.stop_words.contains(t))
                    && Some(t) != self.unknown_words_replacement_string.as_ref()
            })
            .collect();
        let num_tokens = filtered_tokens.len();
        filtered_tokens
            .iter()
            .enumerate()
            .flat_map(|(i, t)| {
                let max_index = self.window_size.map_or(num_tokens, |window_size| {
                    min(i + window_size + 1, num_tokens)
                });
                filtered_tokens[i + 1..max_index].iter().map(move |other| {
                    if self.keep_order {
                        (t.clone(), other.clone())
                    } else {
                        if t < other {
                            (t.clone(), other.clone())
                        } else {
                            (other.clone(), t.clone())
                        }
                    }
                })
            })
            .collect()
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

fn get_word_clusters(query_tokens: &[String], word_clusterer: Arc<WordClusterer>) -> Vec<String> {
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

    use maplit::{hashmap, hashset};
    use ndarray::array;
    use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind};
    use snips_nlu_ontology::{NumberValue, SlotValue};
    use snips_nlu_utils::language::Language;
    use snips_nlu_utils::token::tokenize_light;

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::models::{
        CooccurrenceVectorizerConfiguration, CooccurrenceVectorizerModel, SklearnVectorizerModel,
        TfidfVectorizerConfiguration, TfidfVectorizerModel,
    };
    use crate::resources::stemmer::HashMapStemmer;
    use crate::resources::word_clusterer::HashMapWordClusterer;
    use crate::resources::SharedResources;
    use crate::testutils::assert_epsilon_eq_array1;
    use crate::testutils::MockedBuiltinEntityParser;
    use crate::testutils::MockedCustomEntityParser;

    use super::*;

    #[test]
    fn transform_works() {
        // Given
        let mocked_custom_parser = MockedCustomEntityParser::from_iter(vec![(
            "hello this bird is a beauti bird with 22 wings".to_string(),
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
        let mocked_builtin_parser = MockedBuiltinEntityParser::from_iter(vec![(
            "Hëllo this bïrd is a beautiful Bïrd with 22 wings".to_string(),
            vec![BuiltinEntity {
                value: "22".to_string(),
                range: 41..43,
                entity: SlotValue::Number(NumberValue { value: 22.0 }),
                entity_kind: BuiltinEntityKind::Number,
            }],
        )]);
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
            "entityfeaturegreeting".to_string() => 9,
            "builtinentityfeaturesnipsnumber".to_string() => 10,
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
            1.0,
        ];

        let language_code = "en".to_string();

        let tfidf_vectorizer_ = SklearnVectorizerModel { idf_diag, vocab };

        let tfidf_vectorizer_config = TfidfVectorizerConfiguration {
            use_stemming: true,
            word_clusters_name: None,
        };

        let tfidf_vectorizer_model = TfidfVectorizerModel {
            language_code,
            builtin_entity_scope: vec!["snips/number".to_string()],
            vectorizer: tfidf_vectorizer_,
            config: tfidf_vectorizer_config,
        };

        let tfidf_vectorizer =
            TfidfVectorizer::new(tfidf_vectorizer_model, Arc::new(resources)).unwrap();

        let cooccurrence_vectorizer = None;

        let featurizer = Featurizer {
            tfidf_vectorizer,
            cooccurrence_vectorizer,
        };

        // When
        let input = "Hëllo this bïrd is a beautiful Bïrd with 22 wings";
        let features = featurizer.transform(input).unwrap();

        // Then
        let expected_features = array![
            0.0,
            0.4022979853278378,
            0.5570317855419762,
            0.0,
            0.3298901029212854,
            0.0,
            0.0,
            0.25001173551567585,
            0.30358567884046356,
            0.4821654899230893,
            0.17857981108262563
        ];

        assert_epsilon_eq_array1(&expected_features, &features, 1e-6);
    }

    #[test]
    fn transform_works_with_cooccurrence() {
        // Given
        let mocked_custom_parser = MockedCustomEntityParser::from_iter(vec![
            (
                "hello this bird is a beautiful bird with 22 wings".to_string(),
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
            ),
            (
                "hello this bird is a beauti bird with 22 wings".to_string(),
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
                        range: 28..32,
                        entity_identifier: "animal".to_string(),
                    },
                ],
            ),
        ]);
        let mocked_builtin_parser = MockedBuiltinEntityParser::from_iter(vec![(
            "hello this bird is a beautiful bird with 22 wings".to_string(),
            vec![BuiltinEntity {
                value: "22".to_string(),
                range: 41..43,
                entity: SlotValue::Number(NumberValue { value: 22.0 }),
                entity_kind: BuiltinEntityKind::Number,
            }],
        )]);
        let mocked_stemmer =
            HashMapStemmer::from_iter(vec![("beautiful".to_string(), "beauti".to_string())]);

        let stop_words = hashset!["a".to_string()];

        let resources = Arc::new(SharedResources {
            custom_entity_parser: Arc::new(mocked_custom_parser),
            builtin_entity_parser: Arc::new(mocked_builtin_parser),
            stemmer: Some(Arc::new(mocked_stemmer)),
            word_clusterers: HashMap::new(),
            gazetteers: HashMap::new(),
            stop_words,
        });

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
            "entityfeaturegreeting".to_string() => 9,
            "builtinentityfeaturesnipsnumber".to_string() => 10,
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
            1.0,
        ];

        let word_pairs = hashmap![
            0 => ("ANIMAL".to_string(), "beautiful".to_string()),
            1 => ("hello".to_string(), "ANIMAL".to_string()),
            2 => ("hello".to_string(), "is".to_string()),
            3 => ("this".to_string(), "beautiful".to_string()),
            4 => ("with".to_string(), "SNIPSNUMBER".to_string()),
            5 => ("hello".to_string(), "wings".to_string()),
        ];

        let language_code = "en".to_string();

        let tfidf_vectorizer_ = SklearnVectorizerModel { idf_diag, vocab };

        let tfidf_vectorizer_config = TfidfVectorizerConfiguration {
            use_stemming: true,
            word_clusters_name: None,
        };

        let tfidf_vectorizer_model = TfidfVectorizerModel {
            language_code: language_code.clone(),
            builtin_entity_scope: vec!["snips/number".to_string()],
            vectorizer: tfidf_vectorizer_,
            config: tfidf_vectorizer_config,
        };

        let tfidf_vectorizer =
            TfidfVectorizer::new(tfidf_vectorizer_model, resources.clone()).unwrap();

        let cooccurrence_vectorizer_config = CooccurrenceVectorizerConfiguration {
            window_size: Some(2),
            filter_stop_words: true,
            keep_order: true,
            unknown_words_replacement_string: None,
        };

        let cooccurrence_vectorizer_model = CooccurrenceVectorizerModel {
            language_code,
            builtin_entity_scope: vec!["snips/number".to_string()],
            word_pairs,
            config: cooccurrence_vectorizer_config,
        };

        let cooccurrence_vectorizer =
            CooccurrenceVectorizer::new(cooccurrence_vectorizer_model, resources).unwrap();

        let featurizer = Featurizer {
            tfidf_vectorizer,
            cooccurrence_vectorizer: Some(cooccurrence_vectorizer),
        };

        // When
        let input = "hello this bird is a beautiful bird with 22 wings";
        let features = featurizer.transform(input).unwrap();

        // Then
        let expected_features = array![
            0.0,
            0.4022979853278378,
            0.5570317855419762,
            0.0,
            0.3298901029212854,
            0.0,
            0.0,
            0.25001173551567585,
            0.30358567884046356,
            0.4821654899230893,
            0.17857981108262563, // last tfidf feature
            1.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0
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
        let augmented_query = get_word_clusters(&query_tokens, Arc::new(word_clusterer));

        // Then
        let expected_augmented_query =
            vec!["cluster_house".to_string(), "cluster_love".to_string()];

        assert_eq!(augmented_query, expected_augmented_query)
    }

    #[test]
    fn extract_word_pairs_works() {
        // Given
        let mocked_custom_parser = MockedCustomEntityParser {
            mocked_outputs: hashmap!(),
        };

        let mocked_builtin_parser = MockedBuiltinEntityParser {
            mocked_outputs: hashmap!(),
        };

        let resources = Arc::new(SharedResources {
            custom_entity_parser: Arc::new(mocked_custom_parser),
            builtin_entity_parser: Arc::new(mocked_builtin_parser),
            stemmer: None,
            word_clusterers: HashMap::new(),
            gazetteers: HashMap::new(),
            stop_words: hashset!(),
        });
        let config = CooccurrenceVectorizerConfiguration {
            window_size: None,
            filter_stop_words: false,
            keep_order: true,
            unknown_words_replacement_string: Some("d".to_string()),
        };

        let word_pairs = hashmap!(
            0 => ("a".to_string(), "c".to_string()),
            1 => ("a".to_string(), "b".to_string()),
            2 => ("c".to_string(), "b".to_string()),
        );

        let model = CooccurrenceVectorizerModel {
            language_code: "en".to_string(),
            builtin_entity_scope: vec![],
            word_pairs,
            config,
        };

        let vectorizer = CooccurrenceVectorizer::new(model, resources).unwrap();

        let tokens = vec![
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
            "d".to_string(),
        ];

        // When
        let pairs = vectorizer.extract_word_pairs(tokens);

        // Then
        let expected_pairs = hashset!(
            ("a".to_string(), "c".to_string()),
            ("a".to_string(), "b".to_string()),
            ("c".to_string(), "b".to_string()),
        );
        assert_eq!(expected_pairs, pairs)
    }

    #[test]
    fn extract_word_pairs_unordered_works() {
        // Given
        let mocked_custom_parser = MockedCustomEntityParser {
            mocked_outputs: hashmap!(),
        };

        let mocked_builtin_parser = MockedBuiltinEntityParser {
            mocked_outputs: hashmap!(),
        };

        let resources = Arc::new(SharedResources {
            custom_entity_parser: Arc::new(mocked_custom_parser),
            builtin_entity_parser: Arc::new(mocked_builtin_parser),
            stemmer: None,
            word_clusterers: HashMap::new(),
            gazetteers: HashMap::new(),
            stop_words: hashset!(),
        });
        let config = CooccurrenceVectorizerConfiguration {
            window_size: None,
            filter_stop_words: false,
            keep_order: false,
            unknown_words_replacement_string: Some("d".to_string()),
        };

        let word_pairs = hashmap!(
            0 => ("a".to_string(), "c".to_string()),
            1 => ("a".to_string(), "b".to_string()),
            2 => ("b".to_string(), "c".to_string()),
        );

        let model = CooccurrenceVectorizerModel {
            language_code: "en".to_string(),
            builtin_entity_scope: vec![],
            word_pairs,
            config,
        };

        let vectorizer = CooccurrenceVectorizer::new(model, resources).unwrap();

        let tokens = vec![
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
            "d".to_string(),
        ];

        // When
        let pairs = vectorizer.extract_word_pairs(tokens);

        // Then
        let expected_pairs = hashset!(
            ("a".to_string(), "c".to_string()),
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        );
        assert_eq!(expected_pairs, pairs)
    }
}
