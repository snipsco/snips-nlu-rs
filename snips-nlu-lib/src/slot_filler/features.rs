use itertools::Itertools;

use builtin_entity_parsing::CachingBuiltinEntityParser;
use super::crf_utils::{get_scheme_prefix, TaggingScheme};
use super::features_utils::{get_word_chunk, initial_string_from_tokens};
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::{get_shape, normalize};
use nlu_utils::token::{compute_all_ngrams, Token};
use resources::gazetteer::Gazetteer;
use resources::stemmer::Stemmer;
use resources::word_clusterer::WordClusterer;
use snips_nlu_ontology::BuiltinEntityKind;

pub fn is_digit(string: &str) -> Option<String> {
    if string.chars().all(|c| c.is_digit(10)) {
        Some("1".to_string())
    } else {
        None
    }
}

pub fn length(string: &str) -> Option<String> {
    Some(format!("{:?}", string.chars().count()))
}

pub fn is_first(token_index: usize) -> Option<String> {
    if token_index == 0 {
        Some("1".to_string())
    } else {
        None
    }
}

pub fn is_last(tokens: &[Token], token_index: usize) -> Option<String> {
    if token_index == tokens.len() - 1 {
        Some("1".to_string())
    } else {
        None
    }
}

pub fn prefix(string: &str, prefix_size: usize) -> Option<String> {
    let normalized = normalize(string);
    get_word_chunk(&normalized, prefix_size, 0, false)
}

pub fn suffix(string: &str, suffix_size: usize) -> Option<String> {
    let normalized = normalize(string);
    let chunk_start = normalized.chars().count();
    get_word_chunk(&normalized, suffix_size, chunk_start, true)
}

pub fn shape(tokens: &[Token], token_index: usize, ngram_size: usize) -> Option<String> {
    let max_len = tokens.len();
    let end = token_index + ngram_size;
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

pub fn ngram<S: Stemmer, G: Gazetteer>(
    tokens: &[Token],
    token_index: usize,
    ngram_size: usize,
    stemmer: Option<&S>,
    common_words_gazetteer: Option<&G>,
) -> Option<String> {
    // TODO we should precompute the lowercase value somewhere, perhaps use NormalizedToken ?
    if token_index + ngram_size > tokens.len() {
        None
    } else {
        Some(
            tokens[token_index..token_index + ngram_size]
                .iter()
                .map(|token| {
                    let normalized_value = normalize(&token.value);
                    let stemmed_value = stemmer
                        .map_or(normalized_value.to_string(),
                                |s| s.stem(&normalized_value));
                    common_words_gazetteer
                        .map_or(stemmed_value.clone(),
                                |g| if g.contains(&stemmed_value) {
                                    stemmed_value.clone()
                                } else {
                                    "rare_word".to_string()
                                })
                })
                .join(" "),
        )
    }
}

pub fn get_gazetteer_match<S: Stemmer, G: Gazetteer>(
    tokens: &[Token],
    token_index: usize,
    gazetteer: &G,
    stemmer: Option<&S>,
    tagging_scheme: TaggingScheme,
) -> Option<String> {
    let normalized_tokens = normalize_tokens(tokens, stemmer);
    let normalized_tokens_ref = normalized_tokens.iter().map(|t| &**t).collect_vec();
    let mut filtered_ngrams =
        compute_all_ngrams(&*normalized_tokens_ref, normalized_tokens_ref.len())
            .into_iter()
            .filter(|ngram_indexes| ngram_indexes.1.iter().any(|index| *index == token_index))
            .collect_vec();

    filtered_ngrams.sort_by_key(|ngrams| -(ngrams.1.len() as i64));

    filtered_ngrams
        .iter()
        .find(|ngrams| gazetteer.contains(&ngrams.0))
        .map(|ngrams| get_scheme_prefix(token_index, &ngrams.1, tagging_scheme).to_string())
}

pub fn get_word_cluster<C: WordClusterer>(
    tokens: &[Token],
    token_index: usize,
    word_clusterer: &C,
) -> Option<String> {
    if token_index >= tokens.len() {
        return None;
    }
    word_clusterer.get_cluster(&tokens[token_index].value.to_lowercase())
}

pub fn get_builtin_entity_match(
    tokens: &[Token],
    token_index: usize,
    parser: &CachingBuiltinEntityParser,
    builtin_entity_kind: BuiltinEntityKind,
    tagging_scheme: TaggingScheme,
) -> Option<String> {
    if token_index >= tokens.len() {
        return None;
    }
    let text = initial_string_from_tokens(tokens);
    parser
        .extract_entities(&text, Some(&[builtin_entity_kind]), true)
        .into_iter()
        .find(|e| ranges_overlap(&e.range, &tokens[token_index].char_range))
        .map(|e| {
            let entity_token_indexes = (0..tokens.len())
                .filter(|i| ranges_overlap(&tokens[*i].char_range, &e.range))
                .collect_vec();
            get_scheme_prefix(token_index, &entity_token_indexes, tagging_scheme).to_string()
        })
}

fn normalize_tokens<S: Stemmer>(tokens: &[Token], stemmer: Option<&S>) -> Vec<String> {
    tokens
        .iter()
        .map(|t| stemmer.map_or(normalize(&t.value), |s| s.stem(&normalize(&t.value))))
        .collect_vec()
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
        let parser = CachingBuiltinEntityParser::new(Language::EN, 100);

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
