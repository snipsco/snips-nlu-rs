use std::collections::HashMap;
use std::sync::Arc;

use itertools::Itertools;

use builtin_entity_parsing::CachingBuiltinEntityParser;
use super::crf_utils::{get_scheme_prefix, TaggingScheme};
use errors::*;
use super::features_utils::{get_word_chunk, initial_string_from_tokens};
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::{get_shape, normalize};
use nlu_utils::token::{compute_all_ngrams, Token};
use resources::gazetteer::{Gazetteer, HashSetGazetteer};
use resources::SharedResources;
use resources::stemmer::{HashMapStemmer, Stemmer};
use resources::word_clusterer::WordClusterer;
use slot_filler::feature_processor::{Feature, FeatureKindRepr};
use snips_nlu_ontology::BuiltinEntityKind;

pub struct IsDigitFeature {}

impl Feature for IsDigitFeature {
    fn build_features(
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        if tokens[token_index].value.chars().all(|c| c.is_digit(10)) {
            Some("1".to_string())
        } else {
            None
        }
    }
}

pub struct LengthFeature {}

impl Feature for LengthFeature {
    fn build_features(
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        Some(format!("{:?}", &tokens[token_index].value.chars().count()))
    }
}

pub struct IsFirstFeature {}

impl Feature for IsFirstFeature {
    fn build_features(
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, _tokens: &[Token], token_index: usize) -> Option<String> {
        if token_index == 0 {
            Some("1".to_string())
        } else {
            None
        }
    }
}

pub struct IsLastFeature {}

impl Feature for IsLastFeature {
    fn build_features(
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        if token_index == tokens.len() - 1 {
            Some("1".to_string())
        } else {
            None
        }
    }
}

pub struct NgramFeature {
    ngram_size: usize,
    opt_common_words_gazetteer: Option<Arc<HashSetGazetteer>>,
    opt_stemmer: Option<Arc<HashMapStemmer>>,
}

impl Feature for NgramFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.ngram_size)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let n = parse_as_u64(args, "n")? as usize;
        let common_words_gazetteer_name = parse_as_opt_string(args, "common_words_gazetteer_name")?;
        let opt_common_words_gazetteer = if let Some(gazetteer_name) = common_words_gazetteer_name {
            Some(shared_resources.gazetteers.get(&gazetteer_name)
                .map(|gazetteer| gazetteer.clone())
                .ok_or_else(||
                    format_err!("Cannot find gazetteer '{}' in shared resources", gazetteer_name))?)
        } else {
            None
        };
        let use_stemming = parse_as_bool(args, "use_stemming")?;
        let opt_stemmer = if use_stemming {
            Some(shared_resources.stemmer.as_ref()
                .map(|stemmer| stemmer.clone())
                .ok_or_else(|| format_err!("Cannot find stemmer in shared resources"))?)
        } else {
            None
        };
        Ok(vec![Box::new(
            Self {
                ngram_size: n,
                opt_common_words_gazetteer,
                opt_stemmer,
            }
        )])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        // TODO we should precompute the lowercase value somewhere, perhaps use NormalizedToken ?
        if token_index + self.ngram_size > tokens.len() {
            return None;
        }
        let result = tokens[token_index..token_index + self.ngram_size]
            .iter()
            .map(|token| {
                let stemmed_value = self.opt_stemmer
                    .as_ref()
                    .map(|stemmer| stemmer.stem(&normalize(&token.value)))
                    .unwrap_or_else(|| normalize(&token.value));
                if let Some(common_words_gazetteer) = self.opt_common_words_gazetteer.as_ref() {
                    if common_words_gazetteer.contains(&stemmed_value) {
                        stemmed_value
                    } else {
                        "rare_word".to_string()
                    }
                } else {
                    stemmed_value
                }
            })
            .join(" ");

        Some(result)
    }
}

pub struct ShapeNgramFeature {
    ngram_size: usize,
}

impl Feature for ShapeNgramFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.ngram_size)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let ngram_size = parse_as_u64(args, "n")? as usize;
        Ok(vec![Box::new(Self { ngram_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let max_len = tokens.len();
        let end = token_index + self.ngram_size;
        if token_index < end && end <= max_len {
            Some(
                tokens[token_index..end]
                    .iter()
                    .map(|token| get_shape(&token.value))
                    .join(" "),
            )
        } else {
            None
        }
    }
}

pub struct PrefixFeature {
    prefix_size: usize,
}

impl Feature for PrefixFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.prefix_size)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let prefix_size = parse_as_u64(args, "prefix_size")? as usize;
        Ok(vec![Box::new(Self { prefix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let normalized = normalize(&tokens[token_index].value);
        get_word_chunk(&normalized, self.prefix_size, 0, false)
    }
}

pub struct SuffixFeature {
    suffix_size: usize,
}

impl Feature for SuffixFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.suffix_size)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let suffix_size = parse_as_u64(args, "suffix_size")? as usize;
        Ok(vec![Box::new(Self { suffix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let normalized = normalize(&tokens[token_index].value);
        let chunk_start = normalized.chars().count();
        get_word_chunk(&normalized, self.suffix_size, chunk_start, true)
    }
}

pub struct EntityMatchFeature {
    entity_name: String,
    entity_values: HashSetGazetteer,
    tagging_scheme: TaggingScheme,
    opt_stemmer: Option<Arc<HashMapStemmer>>,
}

impl Feature for EntityMatchFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), &self.entity_name)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let collections = parse_as_vec_of_vec(args, "collections")?;
        let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
        let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
        let use_stemming = parse_as_bool(args, "use_stemming")?;
        let opt_stemmer = if use_stemming {
            Some(shared_resources.stemmer
                .as_ref()
                .map(|stemmer| stemmer.clone())
                .ok_or_else(||
                    format_err!("Cannot find stemmer in shared resources"))?)
        } else {
            None
        };
        Ok(collections
            .into_iter()
            .map(|(entity_name, values)| {
                let entity_values = HashSetGazetteer::from(values.into_iter());
                Box::new(Self {
                    entity_name,
                    entity_values,
                    tagging_scheme,
                    opt_stemmer: opt_stemmer.clone(),
                }) as Box<_>
            })
            .collect())
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let normalized_tokens = normalize_tokens(
            tokens,
            self.opt_stemmer.as_ref().map(|stemmer| stemmer.as_ref()));
        let normalized_tokens_ref = normalized_tokens.iter().map(|t| &**t).collect_vec();
        let mut filtered_ngrams =
            compute_all_ngrams(&*normalized_tokens_ref, normalized_tokens_ref.len())
                .into_iter()
                .filter(|ngram_indexes| ngram_indexes.1.iter().any(|index| *index == token_index))
                .collect_vec();

        filtered_ngrams.sort_by_key(|ngrams| -(ngrams.1.len() as i64));

        filtered_ngrams
            .iter()
            .find(|ngrams| self.entity_values.contains(&ngrams.0))
            .map(|ngrams| get_scheme_prefix(token_index, &ngrams.1, self.tagging_scheme).to_string())
    }
}

pub struct BuiltinEntityMatchFeature {
    tagging_scheme: TaggingScheme,
    builtin_entity_kind: BuiltinEntityKind,
    builtin_entity_parser: Arc<CachingBuiltinEntityParser>,
}

impl Feature for BuiltinEntityMatchFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.builtin_entity_kind.identifier())
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let builtin_entity_labels = parse_as_vec_string(args, "entity_labels")?;
        let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
        let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;

        builtin_entity_labels
            .into_iter()
            .map(|label| {
                let builtin_entity_kind = BuiltinEntityKind::from_identifier(&label)?;
                Ok(Box::new(Self {
                    tagging_scheme,
                    builtin_entity_kind,
                    builtin_entity_parser: shared_resources.builtin_entity_parser.clone(),
                }) as Box<_>)
            })
            .collect()
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let text = initial_string_from_tokens(tokens);
        self.builtin_entity_parser
            .extract_entities(&text, Some(&[self.builtin_entity_kind]), true)
            .into_iter()
            .find(|e| ranges_overlap(&e.range, &tokens[token_index].char_range))
            .map(|e| {
                let entity_token_indexes = (0..tokens.len())
                    .filter(|i| ranges_overlap(&tokens[*i].char_range, &e.range))
                    .collect_vec();
                get_scheme_prefix(token_index, &entity_token_indexes, self.tagging_scheme).to_string()
            })
    }
}

pub struct WordClusterFeature {
    cluster_name: String,
    word_clusterer: Arc<WordClusterer>,
}

impl Feature for WordClusterFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.cluster_name)
    }

    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let cluster_name = parse_as_string(args, "cluster_name")?;
        let word_clusterer = shared_resources.word_clusterers
            .get(&cluster_name)
            .map(|clusterer| clusterer.clone())
            .ok_or_else(|| format_err!(
                "Cannot find word clusters '{}' in shared resources", cluster_name))?;
        Ok(vec![Box::new(Self {
            cluster_name,
            word_clusterer,
        })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        self.word_clusterer.get_cluster(&tokens[token_index].value.to_lowercase())
    }
}

fn normalize_tokens<S: Stemmer>(tokens: &[Token], stemmer: Option<&S>) -> Vec<String> {
    tokens
        .iter()
        .map(|t| stemmer.map_or(normalize(&t.value), |s| s.stem(&normalize(&t.value))))
        .collect_vec()
}

fn parse_as_string(args: &HashMap<String, ::serde_json::Value>, arg_name: &str) -> Result<String> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .ok_or_else(|| format_err!("'{}' isn't a string", arg_name))?
        .to_string())
}

fn parse_as_opt_string(
    args: &HashMap<String, ::serde_json::Value>,
    arg_name: &str,
) -> Result<Option<String>> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .map(|s| s.to_string()))
}

fn parse_as_vec_string(
    args: &HashMap<String, ::serde_json::Value>,
    arg_name: &str,
) -> Result<Vec<String>> {
    args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_array()
        .ok_or_else(|| format_err!("'{}' isn't an array", arg_name))?
        .iter()
        .map(|v| {
            Ok(v.as_str()
                .ok_or_else(|| format_err!("'{}' is not a string", v))?
                .to_string())
        })
        .collect()
}

fn parse_as_vec_of_vec(
    args: &HashMap<String, ::serde_json::Value>,
    arg_name: &str,
) -> Result<Vec<(String, Vec<String>)>> {
    args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_object()
        .ok_or_else(|| format_err!("'{}' isn't a map", arg_name))?
        .into_iter()
        .map(|(k, v)| {
            let values: Result<Vec<_>> = v.as_array()
                .ok_or_else(|| format_err!("'{}' is not a vec", v))?
                .into_iter()
                .map(|item| {
                    Ok(item.as_str()
                        .ok_or_else(|| format_err!("'{}' is not a string", item))?
                        .to_string())
                })
                .collect();
            Ok((k.to_string(), values?))
        })
        .collect()
}

fn parse_as_bool(args: &HashMap<String, ::serde_json::Value>, arg_name: &str) -> Result<bool> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_bool()
        .ok_or_else(|| format_err!("'{}' isn't a bool", arg_name))?)
}

fn parse_as_u64(args: &HashMap<String, ::serde_json::Value>, arg_name: &str) -> Result<u64> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_u64()
        .ok_or_else(|| format_err!("'{}' isn't a u64", arg_name))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::From;

    use nlu_utils::language::Language as NluUtilsLanguage;
    use nlu_utils::token::tokenize;
    use snips_nlu_ontology::Language;
    use resources::stemmer::HashMapStemmer;
    use resources::gazetteer::HashSetGazetteer;
    use resources::word_clusterer::HashMapWordClusterer;

    #[test]
    fn is_digit_feature_works() {
        // Given
        let tokens = tokenize("e3 abc 42 5r", NluUtilsLanguage::EN);
        let feature = IsDigitFeature {};

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![None, None, Some("1".to_string()), None];
        assert_eq!(expected_results, results)
    }

    #[test]
    fn length_feature_works() {
        // Given
        let tokens = tokenize("hello world helloworld", NluUtilsLanguage::EN);
        let feature = LengthFeature {};

        // When
        let results: Vec<Option<String>> = (0..3).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            Some("5".to_string()),
            Some("5".to_string()),
            Some("10".to_string()),
        ];

        assert_eq!(expected_results, results);
    }

    #[test]
    fn prefix_feature_works() {
        // Given
        let tokens = tokenize("hello_world foo_bar", NluUtilsLanguage::EN);
        let feature = PrefixFeature { prefix_size: 6 };

        // When
        let actual_result: Vec<Option<String>> = (0..2).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_result = vec![Some("hello_".to_string()), Some("foo_ba".to_string())];
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn suffix_feature_works() {
        // Given
        let tokens = tokenize("hello_world foo_bar", NluUtilsLanguage::EN);
        let feature = SuffixFeature { suffix_size: 6 };

        // When
        let actual_result: Vec<Option<String>> = (0..2).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_result = vec![Some("_world".to_string()), Some("oo_bar".to_string())];
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn shape_feature_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("Hello BEAUTIFUL world !!!", language);
        let feature = ShapeNgramFeature { ngram_size: 2 };

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_result = vec![
            Some("Xxx XXX".to_string()),
            Some("XXX xxx".to_string()),
            Some("xxx xX".to_string()),
            None
        ];
        assert_eq!(expected_result, results);
    }

    #[test]
    fn ngram_feature_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);
        let feature = NgramFeature {
            ngram_size: 2,
            opt_common_words_gazetteer: None,
            opt_stemmer: None,
        };

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            Some("i love".to_string()),
            Some("love house".to_string()),
            Some("house music".to_string()),
            None
        ];

        assert_eq!(expected_results, results);
    }

    #[test]
    fn ngram_feature_works_with_common_words_gazetteer() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);
        let common_words_gazetteer = HashSetGazetteer::from(
            vec!["i".to_string(), "love".to_string(), "music".to_string()].into_iter(),
        );
        let feature = NgramFeature {
            ngram_size: 2,
            opt_common_words_gazetteer: Some(Arc::new(common_words_gazetteer)),
            opt_stemmer: None,
        };

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            Some("i love".to_string()),
            Some("love rare_word".to_string()),
            Some("rare_word music".to_string()),
            None
        ];
        assert_eq!(expected_results, results);
    }

    #[test]
    fn ngram_feature_works_with_stemmer() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);
        let stemmer = HashMapStemmer::from(
            vec![("house".to_string(), "hous".to_string())].into_iter()
        );
        let feature = NgramFeature {
            ngram_size: 2,
            opt_common_words_gazetteer: None,
            opt_stemmer: Some(Arc::new(stemmer)),
        };

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            Some("i love".to_string()),
            Some("love hous".to_string()),
            Some("hous music".to_string()),
            None,
        ];

        assert_eq!(expected_results, results);
    }

    #[test]
    fn entity_match_feature_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let gazetteer = HashSetGazetteer::from(
            vec![
                "bird".to_string(),
                "blue bird".to_string(),
                "beautiful blue bird".to_string(),
            ].into_iter(),
        );
        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love this beautiful blue Bird !", language);
        let feature = EntityMatchFeature {
            entity_name: "bird_type".to_string(),
            entity_values: gazetteer,
            tagging_scheme,
            opt_stemmer: None,
        };

        // When
        let results: Vec<Option<String>> = (0..6).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            None,
            None,
            None,
            Some("B-".to_string()),
            Some("I-".to_string()),
            Some("L-".to_string()),
        ];
        assert_eq!(expected_results, results);
    }

    #[test]
    fn entity_match_feature_works_with_stemming() {
        // Given
        let language = NluUtilsLanguage::EN;
        let stemmer = HashMapStemmer::from(
            vec![("birds".to_string(), "bird".to_string())].into_iter()
        );
        let gazetteer = HashSetGazetteer::from(
            vec![
                "bird".to_string(),
                "blue bird".to_string(),
                "beautiful blue bird".to_string(),
            ].into_iter(),
        );

        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love Blue Birds !", language);
        let feature = EntityMatchFeature {
            entity_name: "bird_type".to_string(),
            entity_values: gazetteer,
            tagging_scheme,
            opt_stemmer: Some(Arc::new(stemmer)),
        };

        // When
        let results: Vec<Option<String>> = (0..5).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            None,
            None,
            Some("B-".to_string()),
            Some("L-".to_string()),
            None
        ];
        assert_eq!(expected_results, results);
    }

    #[test]
    fn builtin_entity_match_feature_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("Let's meet tomorrow at 9pm ok ?", language);
        let tagging_scheme = TaggingScheme::BILOU;
        let parser = CachingBuiltinEntityParser::from_language(Language::EN, 100).unwrap();
        let feature = BuiltinEntityMatchFeature {
            tagging_scheme,
            builtin_entity_kind: BuiltinEntityKind::Time,
            builtin_entity_parser: Arc::new(parser),
        };

        // When
        let results: Vec<Option<String>> = (0..7).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            None,
            None,
            None,
            Some("B-".to_string()),
            Some("I-".to_string()),
            Some("L-".to_string()),
            None
        ];
        assert_eq!(expected_results, results);
    }

    #[test]
    fn word_cluster_feature_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let word_clusterer = HashMapWordClusterer::from(
            vec![("bird".to_string(), "010101".to_string())].into_iter()
        );
        let tokens = tokenize("I love this bird", language);
        let feature = WordClusterFeature {
            cluster_name: "test_clusters".to_string(),
            word_clusterer: Arc::new(word_clusterer),
        };

        // When
        let results: Vec<Option<String>> = (0..4).map(|i| feature.compute(&tokens, i)).collect();

        // Then
        let expected_results = vec![
            None,
            None,
            None,
            Some("010101".to_string())
        ];
        assert_eq!(expected_results, results);
    }
}
