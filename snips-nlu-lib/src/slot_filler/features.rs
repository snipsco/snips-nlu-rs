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
use slot_filler::feature_processor::Feature;
use snips_nlu_ontology::BuiltinEntityKind;


pub struct IsDigitFeature {
    offsets: Vec<i32>
}

impl Feature for IsDigitFeature {
    fn base_name(&self) -> &'static str {
        "is_digit"
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self { offsets: offsets.to_vec() })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        if tokens[token_index].value.chars().all(|c| c.is_digit(10)) {
            Some("1".to_string())
        } else {
            None
        }
    }
}

pub struct LengthFeature {
    offsets: Vec<i32>
}

impl Feature for LengthFeature {
    fn base_name(&self) -> &'static str {
        "length"
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self { offsets: offsets.to_vec() })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        Some(format!("{:?}", &tokens[token_index].value.chars().count()))
    }
}

pub struct IsFirstFeature {
    offsets: Vec<i32>
}

impl Feature for IsFirstFeature {
    fn base_name(&self) -> &'static str {
        "is_first"
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self { offsets: offsets.to_vec() })])
    }

    fn compute(&self, _tokens: &[Token], token_index: usize) -> Option<String> {
        if token_index == 0 {
            Some("1".to_string())
        } else {
            None
        }
    }
}

pub struct IsLastFeature {
    offsets: Vec<i32>
}

impl Feature for IsLastFeature {
    fn base_name(&self) -> &'static str {
        "is_last"
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        _args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        Ok(vec![Box::new(Self { offsets: offsets.to_vec() })])
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
    offsets: Vec<i32>,
    opt_common_words_gazetteer: Option<Arc<HashSetGazetteer>>,
    opt_stemmer: Option<Arc<HashMapStemmer>>,
}

impl Feature for NgramFeature {
    fn base_name(&self) -> &'static str {
        "ngram"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.ngram_size)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
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
                offsets: offsets.to_vec(),
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
    offsets: Vec<i32>,
    ngram_size: usize,
}

impl Feature for ShapeNgramFeature {
    fn base_name(&self) -> &'static str {
        "shape_ngram"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.ngram_size)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let ngram_size = parse_as_u64(args, "n")? as usize;
        Ok(vec![Box::new(Self { offsets: offsets.to_vec(), ngram_size })])
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
    offsets: Vec<i32>,
    prefix_size: usize,
}

impl Feature for PrefixFeature {
    fn base_name(&self) -> &'static str {
        "prefix"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.prefix_size)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let prefix_size = parse_as_u64(args, "prefix_size")? as usize;
        Ok(vec![Box::new(Self { offsets: offsets.to_vec(), prefix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let normalized = normalize(&tokens[token_index].value);
        get_word_chunk(&normalized, self.prefix_size, 0, false)
    }
}

pub struct SuffixFeature {
    offsets: Vec<i32>,
    suffix_size: usize,
}

impl Feature for SuffixFeature {
    fn base_name(&self) -> &'static str {
        "suffix"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.suffix_size)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
        args: &HashMap<String, ::serde_json::Value>,
        _shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> {
        let suffix_size = parse_as_u64(args, "suffix_size")? as usize;
        Ok(vec![Box::new(Self { offsets: offsets.to_vec(), suffix_size })])
    }

    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String> {
        let normalized = normalize(&tokens[token_index].value);
        let chunk_start = normalized.chars().count();
        get_word_chunk(&normalized, self.suffix_size, chunk_start, true)
    }
}

pub struct EntityMatchFeature {
    offsets: Vec<i32>,
    entity_name: String,
    entity_values: HashSetGazetteer,
    tagging_scheme: TaggingScheme,
    opt_stemmer: Option<Arc<HashMapStemmer>>,
}

impl Feature for EntityMatchFeature {
    fn base_name(&self) -> &'static str {
        "entity_match"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), &self.entity_name)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
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
                    offsets: offsets.to_vec(),
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
    offsets: Vec<i32>,
}

impl Feature for BuiltinEntityMatchFeature {
    fn base_name(&self) -> &'static str {
        "builtin_entity_match"
    }
    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.builtin_entity_kind.identifier())
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
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
                    offsets: offsets.to_vec(),
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
    offsets: Vec<i32>,
    cluster_name: String,
    word_clusterer: Arc<WordClusterer>,
}

impl Feature for WordClusterFeature {
    fn base_name(&self) -> &'static str {
        "word_cluster"
    }

    fn name(&self) -> String {
        format!("{}_{}", self.base_name(), self.cluster_name)
    }

    fn offsets(&self) -> &[i32] {
        self.offsets.as_ref()
    }

    fn build_features(
        offsets: &[i32],
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
            offsets: offsets.to_vec(),
            cluster_name,
            word_clusterer
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

    use nlu_utils::language::Language as NluUtilsLanguage;
    use nlu_utils::token::tokenize;
    use snips_nlu_ontology::Language;
    use resources::stemmer::HashMapStemmer;
    use resources::gazetteer::HashSetGazetteer;

    #[test]
    fn is_digit_works() {
        // Given
        let inputs = vec!["e3", "abc", "42", "5r"];

        // When
        let results: Vec<Option<String>> = (0..4).map(|index| is_digit(inputs[index])).collect();

        // Then
        let expected_results = vec![None, None, Some("1".to_string()), None];
        assert_eq!(results, expected_results)
    }

    #[test]
    fn length_works() {
        // Given
        let inputs = vec!["hello", "こんにちは", "hello こんにちは", ""];

        // When
        let results: Vec<Option<String>> = inputs.iter().map(|s| length(s)).collect();

        // Then
        let expected_lengths = vec![
            Some("5".to_string()),
            Some("5".to_string()),
            Some("11".to_string()),
            Some("0".to_string()),
        ];

        assert_eq!(expected_lengths, results);
    }

    #[test]
    fn prefix_works() {
        // Given
        let string = "hello_world";

        // When
        let actual_result = prefix(string, 6);

        // Then
        let expected_result = Some("hello_".to_string());
        assert_eq!(actual_result, expected_result)
    }

    #[test]
    fn suffix_works() {
        // Given
        let string = "hello_world";

        // When
        let actual_result = suffix(string, 6);

        // Then
        let expected_result = Some("_world".to_string());
        assert_eq!(actual_result, expected_result)
    }

    #[test]
    fn shape_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("Hello BEAUTIFUL world !!!", language);

        // When
        let actual_result = vec![shape(&tokens, 0, 2), shape(&tokens, 1, 3)];

        // Then
        let expected_result = vec![Some("Xxx XXX".to_string()), Some("XXX xxx xX".to_string())];
        assert_eq!(actual_result, expected_result)
    }

    fn assert_ngrams_eq<S: Stemmer, G: Gazetteer>(
        expected_ngrams: Vec<Vec<Option<String>>>,
        tokens: &[Token],
        stemmer: Option<&S>,
        gazetteer: Option<&G>,
    ) {
        for (n, expected_ngrams) in expected_ngrams.iter().enumerate() {
            for (i, expected_ngram) in expected_ngrams.iter().enumerate() {
                let actual_ngrams = ngram(tokens, i, n + 1, stemmer, gazetteer);
                assert_eq!(*expected_ngram, actual_ngrams)
            }
        }
    }

    #[test]
    fn ngram_works() {
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);

        let expected_ngrams = vec![
            vec![
                Some("i".to_string()),
                Some("love".to_string()),
                Some("house".to_string()),
                Some("music".to_string()),
            ],
            vec![
                Some("i love".to_string()),
                Some("love house".to_string()),
                Some("house music".to_string()),
                None,
            ],
            vec![
                Some("i love house".to_string()),
                Some("love house music".to_string()),
                None,
                None,
            ],
        ];

        assert_ngrams_eq(
            expected_ngrams,
            &tokens,
            None as Option<&HashMapStemmer>,
            None as Option<&HashSetGazetteer>,
        );
    }

    #[test]
    fn ngram_works_with_common_words_gazetteer() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);
        let common_words_gazetteer = HashSetGazetteer::from(
            vec!["i".to_string(), "love".to_string(), "music".to_string()].into_iter(),
        );

        // Then
        let expected_ngrams = vec![
            vec![
                Some("i".to_string()),
                Some("love".to_string()),
                Some("rare_word".to_string()),
                Some("music".to_string()),
            ],
            vec![
                Some("i love".to_string()),
                Some("love rare_word".to_string()),
                Some("rare_word music".to_string()),
                None,
            ],
            vec![
                Some("i love rare_word".to_string()),
                Some("love rare_word music".to_string()),
                None,
                None,
            ],
        ];

        assert_ngrams_eq(
            expected_ngrams,
            &tokens,
            None as Option<&HashMapStemmer>,
            Some(&common_words_gazetteer),
        );
    }

    #[test]
    fn ngram_works_with_stemmer() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("I love House Music", language);
        struct TestStemmer;
        impl Stemmer for TestStemmer {
            fn stem(&self, value: &str) -> String {
                if value == "house" {
                    "hous".to_string()
                } else {
                    value.to_string()
                }
            }
        }

        let stemmer = TestStemmer {};

        // Then
        let expected_ngrams = vec![
            vec![
                Some("i".to_string()),
                Some("love".to_string()),
                Some("hous".to_string()),
                Some("music".to_string()),
            ],
            vec![
                Some("i love".to_string()),
                Some("love hous".to_string()),
                Some("hous music".to_string()),
                None,
            ],
            vec![
                Some("i love hous".to_string()),
                Some("love hous music".to_string()),
                None,
                None,
            ],
        ];

        assert_ngrams_eq(
            expected_ngrams,
            &tokens,
            Some(&stemmer),
            None as Option<&HashSetGazetteer>,
        );
    }

    #[test]
    fn get_gazetteer_match_works() {
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
        let token_index = 5;

        // When
        let actual_result = get_gazetteer_match(
            &tokens,
            token_index,
            &gazetteer,
            None as Option<&HashMapStemmer>,
            tagging_scheme,
        );

        // Then
        assert_eq!(Some("L-".to_string()), actual_result)
    }

    #[test]
    fn get_gazetteer_match_works_with_stemming() {
        // Given
        struct TestStemmer;
        impl Stemmer for TestStemmer {
            fn stem(&self, value: &str) -> String {
                if value == "birds" {
                    "bird".to_string()
                } else {
                    value.to_string()
                }
            }
        }

        let language = NluUtilsLanguage::EN;
        let stemmer = TestStemmer {};
        let gazetteer = HashSetGazetteer::from(
            vec![
                "bird".to_string(),
                "blue bird".to_string(),
                "beautiful blue bird".to_string(),
            ].into_iter(),
        );

        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love Blue Birds !", language);
        let token_index = 3;

        // When
        let actual_result = get_gazetteer_match(
            &tokens,
            token_index,
            &gazetteer,
            Some(&stemmer),
            tagging_scheme,
        );

        // Then
        assert_eq!(Some("L-".to_string()), actual_result)
    }

    #[test]
    fn get_builtin_entity_match_works() {
        // Given
        let language = NluUtilsLanguage::EN;
        let tokens = tokenize("Let's meet tomorrow at 9pm ok ?", language);
        let token_index = 5; // 9pm
        let tagging_scheme = TaggingScheme::BILOU;
        let parser = CachingBuiltinEntityParser::from_language(Language::EN, 100).unwrap();

        // When
        let actual_annotation = get_builtin_entity_match(
            &tokens,
            token_index,
            &parser,
            BuiltinEntityKind::Time,
            tagging_scheme,
        );

        // Then
        assert_eq!(Some("L-".to_string()), actual_annotation)
    }

    #[test]
    fn get_word_cluster_works() {
        // Given
        struct TestWordClusterer;
        impl WordClusterer for TestWordClusterer {
            fn get_cluster(&self, word: &str) -> Option<String> {
                if word == "bird" {
                    Some("010101".to_string())
                } else {
                    None
                }
            }
        }

        let language = NluUtilsLanguage::EN;
        let word_clusterer = TestWordClusterer {};
        let tokens = tokenize("I love this bird", language);
        let token_index = 3;

        // When
        let actual_result = get_word_cluster(&tokens, token_index, &word_clusterer);

        // Then
        assert_eq!(Some("010101".to_string()), actual_result);
    }
}
