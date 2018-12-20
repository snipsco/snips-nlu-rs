use std::collections::HashMap;
use std::sync::Arc;

use itertools::Itertools;
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::{get_shape, normalize};
use nlu_utils::token::Token;
use snips_nlu_ontology::BuiltinEntityKind;

use crate::entity_parser::{CustomEntityParser, BuiltinEntityParser};
use crate::errors::*;
use crate::resources::gazetteer::Gazetteer;
use crate::resources::SharedResources;
use crate::resources::stemmer::Stemmer;
use crate::resources::word_clusterer::WordClusterer;

use super::feature_processor::{Feature, FeatureKindRepr};
use super::crf_utils::{get_scheme_prefix, TaggingScheme};
use super::features_utils::{get_word_chunk, initial_string_from_tokens};

pub struct IsDigitFeature {}

impl Feature for IsDigitFeature {
    fn build_features(
        _args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        Ok(if tokens[token_index].value.chars().all(|c| c.is_digit(10)) {
            Some("1".to_string())
        } else {
            None
        })
    }
}

pub struct LengthFeature {}

impl Feature for LengthFeature {
    fn build_features(
        _args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        Ok(Some(format!("{:?}", &tokens[token_index].value.chars().count())))
    }
}

pub struct IsFirstFeature {}

impl Feature for IsFirstFeature {
    fn build_features(
        _args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, _tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        Ok(if token_index == 0 {
            Some("1".to_string())
        } else {
            None
        })
    }
}

pub struct IsLastFeature {}

impl Feature for IsLastFeature {
    fn build_features(
        _args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self {})])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        Ok(if token_index == tokens.len() - 1 {
            Some("1".to_string())
        } else {
            None
        })
    }
}

pub struct NgramFeature {
    ngram_size: usize,
    opt_common_words_gazetteer: Option<Arc<Gazetteer>>,
    opt_stemmer: Option<Arc<Stemmer>>,
}

impl Feature for NgramFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.ngram_size)
    }

    fn build_features(
        args: &HashMap<String, serde_json::Value>,
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
            Some(shared_resources.stemmer
                .clone()
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

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        // TODO we should precompute the lowercase value somewhere, perhaps use NormalizedToken ?
        if token_index + self.ngram_size > tokens.len() {
            return Ok(None);
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

        Ok(Some(result))
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
        args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let ngram_size = parse_as_u64(args, "n")? as usize;
        Ok(vec![Box::new(Self { ngram_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        let max_len = tokens.len();
        let end = token_index + self.ngram_size;
        Ok(if token_index < end && end <= max_len {
            Some(
                tokens[token_index..end]
                    .iter()
                    .map(|token| get_shape(&token.value))
                    .join(" "),
            )
        } else {
            None
        })
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
        args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let prefix_size = parse_as_u64(args, "prefix_size")? as usize;
        Ok(vec![Box::new(Self { prefix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        let normalized = normalize(&tokens[token_index].value);
        Ok(get_word_chunk(&normalized, self.prefix_size, 0, false))
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
        args: &HashMap<String, serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let suffix_size = parse_as_u64(args, "suffix_size")? as usize;
        Ok(vec![Box::new(Self { suffix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        let normalized = normalize(&tokens[token_index].value);
        let chunk_start = normalized.chars().count();
        Ok(get_word_chunk(&normalized, self.suffix_size, chunk_start, true))
    }
}

pub struct CustomEntityMatchFeature {
    entity_name: String,
    tagging_scheme: TaggingScheme,
    opt_stemmer: Option<Arc<Stemmer>>,
    custom_entity_parser: Arc<CustomEntityParser>,
}

impl Feature for CustomEntityMatchFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), &self.entity_name)
    }

    fn build_features(
        args: &HashMap<String, serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let entities = parse_as_vec_string(args, "entities")?;
        let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
        let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
        let use_stemming = parse_as_bool(args, "use_stemming")?;
        let opt_stemmer = if use_stemming {
            Some(shared_resources.stemmer
                .clone()
                .ok_or_else(|| format_err!("Cannot find stemmer in shared resources"))?)
        } else {
            None
        };
        Ok(entities
            .into_iter()
            .map(|entity_name| {
                Box::new(Self {
                    entity_name,
                    tagging_scheme,
                    opt_stemmer: opt_stemmer.clone(),
                    custom_entity_parser: shared_resources.custom_entity_parser.clone(),
                }) as Box<_>
            })
            .collect())
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        let normalized_tokens = transform_tokens(tokens, self.opt_stemmer.clone());
        let normalized_text = initial_string_from_tokens(&*normalized_tokens);

        Ok(self.custom_entity_parser
            .extract_entities(&normalized_text, Some(&[self.entity_name.clone()]))?
            .into_iter()
            .find(|e| ranges_overlap(&e.range, &normalized_tokens[token_index].char_range))
            .map(|e| {
                let entity_token_indexes = (0..normalized_tokens.len())
                    .filter(|i| ranges_overlap(&normalized_tokens[*i].char_range, &e.range))
                    .collect_vec();
                get_scheme_prefix(token_index, &entity_token_indexes, self.tagging_scheme).to_string()
            }))
    }
}

pub struct BuiltinEntityMatchFeature {
    tagging_scheme: TaggingScheme,
    builtin_entity_kind: BuiltinEntityKind,
    builtin_entity_parser: Arc<BuiltinEntityParser>,
}

impl Feature for BuiltinEntityMatchFeature {
    fn name(&self) -> String {
        format!("{}_{}", self.feature_kind().identifier(), self.builtin_entity_kind.identifier())
    }

    fn build_features(
        args: &HashMap<String, serde_json::Value>,
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

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        let text = initial_string_from_tokens(tokens);
        Ok(self.builtin_entity_parser
            .extract_entities(&text, Some(&[self.builtin_entity_kind]), true)?
            .into_iter()
            .find(|e| ranges_overlap(&e.range, &tokens[token_index].char_range))
            .map(|e| {
                let entity_token_indexes = (0..tokens.len())
                    .filter(|i| ranges_overlap(&tokens[*i].char_range, &e.range))
                    .collect_vec();
                get_scheme_prefix(token_index, &entity_token_indexes, self.tagging_scheme).to_string()
            }))
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
        args: &HashMap<String, serde_json::Value>,
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

    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>> {
        Ok(self.word_clusterer.get_cluster(&tokens[token_index].value.to_lowercase()))
    }
}

fn transform_tokens(tokens: &[Token], stemmer: Option<Arc<Stemmer>>) -> Vec<Token> {
    let mut current_char_index = 0;
    let mut current_byte_index = 0;
    tokens
        .iter()
        .map(|t| {
            let normalized_value = stemmer
                .clone()
                .map_or(normalize(&t.value), |s| s.stem(&normalize(&t.value)));
            let char_range = current_char_index..(current_char_index + normalized_value.chars().count());
            let byte_range = current_byte_index..(current_byte_index + normalized_value.len());
            current_char_index = char_range.end + 1;
            current_byte_index = byte_range.end + 1;
            Token::new(normalized_value, byte_range, char_range)
        })
        .collect_vec()
}

fn parse_as_string(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<String> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .ok_or_else(|| format_err!("'{}' isn't a string", arg_name))?
        .to_string())
}

fn parse_as_opt_string(
    args: &HashMap<String, serde_json::Value>,
    arg_name: &str,
) -> Result<Option<String>> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .map(|s| s.to_string()))
}

fn parse_as_vec_string(
    args: &HashMap<String, serde_json::Value>,
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

fn parse_as_bool(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<bool> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_bool()
        .ok_or_else(|| format_err!("'{}' isn't a bool", arg_name))?)
}

fn parse_as_u64(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<u64> {
    Ok(args.get(arg_name)
        .ok_or_else(|| format_err!("can't retrieve '{}' parameter", arg_name))?
        .as_u64()
        .ok_or_else(|| format_err!("'{}' isn't a u64", arg_name))?)
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use snips_nlu_ontology::{BuiltinEntity, SlotValue, TemperatureValue};
    use nlu_utils::language::Language as NluUtilsLanguage;
    use nlu_utils::token::tokenize;

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::resources::gazetteer::HashSetGazetteer;
    use crate::resources::stemmer::HashMapStemmer;
    use crate::resources::word_clusterer::HashMapWordClusterer;
    use crate::testutils::{MockedBuiltinEntityParser, MockedCustomEntityParser};
    use super::*;

    #[test]
    fn transform_tokens_should_work() {
        // Given
        let tokens = tokenize("fo£ root_suffix inflection bar", NluUtilsLanguage::EN);
        let stemmer = HashMapStemmer::from_iter(
            vec![
                ("root_suffix".to_string(), "root".to_string()),
                ("inflection".to_string(), "original_word".to_string())
            ]
        );

        // When
        let transformed_tokens = transform_tokens(&tokens, Some(Arc::new(stemmer)));

        // Then
        let expected_tokens = vec![
            Token::new(
                "fo".to_string(),
                0..2,
                0..2,
            ),
            Token::new(
                "£".to_string(),
                3..5,
                3..4,
            ),
            Token::new(
                "root".to_string(),
                6..10,
                5..9,
            ),
            Token::new(
                "original_word".to_string(),
                11..24,
                10..23,
            ),
            Token::new(
                "bar".to_string(),
                25..28,
                24..27,
            )
        ];
        assert_eq!(expected_tokens, transformed_tokens);
    }

    #[test]
    fn is_digit_feature_works() {
        // Given
        let tokens = tokenize("e3 abc 42 5r", NluUtilsLanguage::EN);
        let feature = IsDigitFeature {};

        // When
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let results: Vec<Option<String>> = (0..3)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let actual_result: Vec<Option<String>> = (0..2)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let actual_result: Vec<Option<String>> = (0..2)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let common_words_gazetteer = HashSetGazetteer::from_iter(
            vec!["i".to_string(), "love".to_string(), "music".to_string()].into_iter(),
        );
        let feature = NgramFeature {
            ngram_size: 2,
            opt_common_words_gazetteer: Some(Arc::new(common_words_gazetteer)),
            opt_stemmer: None,
        };

        // When
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let stemmer = HashMapStemmer::from_iter(
            vec![("house".to_string(), "hous".to_string())].into_iter()
        );
        let feature = NgramFeature {
            ngram_size: 2,
            opt_common_words_gazetteer: None,
            opt_stemmer: Some(Arc::new(stemmer)),
        };

        // When
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let entity_name = "bird_type".to_string();
        let mocked_entity_parser = MockedCustomEntityParser::from_iter(
            vec![(
                "i love this beautiful blue bird !".to_string(),
                vec![
                    CustomEntity {
                        value: "beautiful blue bird".to_string(),
                        resolved_value: "beautiful blue bird".to_string(),
                        range: 12..31,
                        entity_identifier: entity_name.to_string(),
                    }
                ]
            )]
        );
        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love this beautiful blue Bird !", language);
        let feature = CustomEntityMatchFeature {
            entity_name,
            tagging_scheme,
            opt_stemmer: None,
            custom_entity_parser: Arc::new(mocked_entity_parser),
        };

        // When
        let results: Vec<Option<String>> = (0..6)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let stemmer = HashMapStemmer::from_iter(vec![("birds".to_string(), "bird".to_string())]);

        let mocked_entity_parser = MockedCustomEntityParser::from_iter(
            vec![(
                "i love blue bird !".to_string(),
                vec![
                    CustomEntity {
                        value: "blue bird".to_string(),
                        resolved_value: "blue bird".to_string(),
                        range: 7..16,
                        entity_identifier: "bird_type".to_string(),
                    }
                ]
            )]
        );

        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love Blue Birds !", language);
        let feature = CustomEntityMatchFeature {
            entity_name: "bird_type".to_string(),
            tagging_scheme,
            opt_stemmer: Some(Arc::new(stemmer)),
            custom_entity_parser: Arc::new(mocked_entity_parser),
        };

        // When
        let results: Vec<Option<String>> = (0..5)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let input = "Please raise to twenty one degrees ok ?";
        let tokens = tokenize(input, language);
        let tagging_scheme = TaggingScheme::BILOU;
        let mocked_builtin_parser = MockedBuiltinEntityParser::from_iter(
            vec![(
                input.to_string(),
                vec![
                    BuiltinEntity {
                        value: "twenty one degrees".to_string(),
                        range: 16..34,
                        entity: SlotValue::Temperature(TemperatureValue { value: 21.0, unit: None }),
                        entity_kind: BuiltinEntityKind::Temperature,
                    }
                ]
            )]
        );

        let feature = BuiltinEntityMatchFeature {
            tagging_scheme,
            builtin_entity_kind: BuiltinEntityKind::Time,
            builtin_entity_parser: Arc::new(mocked_builtin_parser),
        };

        // When
        let results: Vec<Option<String>> = (0..7)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
        let word_clusterer = HashMapWordClusterer::from_iter(
            vec![("bird".to_string(), "010101".to_string())].into_iter()
        );
        let tokens = tokenize("I love this bird", language);
        let feature = WordClusterFeature {
            cluster_name: "test_clusters".to_string(),
            word_clusterer: Arc::new(word_clusterer),
        };

        // When
        let results: Vec<Option<String>> = (0..4)
            .map(|i| feature.compute(&tokens, i).unwrap())
            .collect();

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
