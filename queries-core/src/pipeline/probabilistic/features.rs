use itertools::Itertools;
use utils::token::{Token, compute_all_ngrams};
use models::gazetteer::Gazetteer;
#[cfg(test)]
use models::gazetteer::HashSetGazetteer;
use models::stemmer::Stemmer;
#[cfg(test)]
use models::stemmer::StaticMapStemmer;
use models::word_clusterer::WordClusterer;
use super::crf_utils::{TaggingScheme, get_scheme_prefix};
use super::features_utils::{get_word_chunk, get_shape, initial_string_from_tokens};
use builtin_entities::{EntityKind, RustlingParser};
use utils::miscellaneous::ranges_overlap;

pub fn is_digit(string: &str) -> Option<String> {
    if string.chars().all(|c| c.is_digit(10)) {
        Some("1".to_string())
    } else {
        None
    }
}

pub fn is_first(token_index: usize) -> Option<String> {
    if token_index == 0 { Some("1".to_string()) } else { None }
}

pub fn is_last(tokens: &[Token], token_index: usize) -> Option<String> {
    if token_index == tokens.len() - 1 {
        Some("1".to_string())
    } else {
        None
    }
}

pub fn prefix(string: &str, prefix_size: usize) -> Option<String> {
    let normalized = string.to_lowercase();
    get_word_chunk(normalized, prefix_size, 0, false)
}

pub fn suffix(string: &str, suffix_size: usize) -> Option<String> {
    let normalized = string.to_lowercase();
    let chunk_start = normalized.chars().count();
    get_word_chunk(normalized, suffix_size, chunk_start, true)
}

pub fn shape(tokens: &[Token], token_index: usize, ngram_size: usize) -> Option<String> {
    let max_len = tokens.len();
    let end = token_index + ngram_size;
    if token_index < end && end <= max_len {
        Some(tokens[token_index..end]
            .iter()
            .map(|token| get_shape(&token.value))
            .join(" ")
        )
    } else {
        None
    }
}


pub fn ngram<S: Stemmer, G: Gazetteer>(tokens: &[Token],
                                       token_index: usize,
                                       ngram_size: usize,
                                       stemmer: Option<&S>,
                                       common_words_gazetteer: Option<&G>) -> Option<String> {
    // TODO we should precompute the lowercase value somewhere, perhaps use NormalizedToken ?
    if token_index + ngram_size > tokens.len() {
        None
    } else {
        Some(
            tokens[token_index..token_index + ngram_size]
                .iter()
                .map(|token| {
                    let lowercased_value = token.value.to_lowercase();
                    let stemmed_value = stemmer
                        .map_or(lowercased_value.to_string(), |s| s.stem(&lowercased_value));
                    common_words_gazetteer
                        .map_or(stemmed_value.clone(), |g|
                            if g.contains(&stemmed_value) {
                                stemmed_value.clone()
                            } else {
                                "rare_word".to_string()
                            }
                        )
                })
                .join(" ")
        )
    }
}

pub fn is_in_gazetteer<S: Stemmer, G: Gazetteer>(tokens: &[Token],
                                                 token_index: usize,
                                                 gazetteer: &G,
                                                 stemmer: Option<&S>,
                                                 tagging_scheme: TaggingScheme) -> Option<String> {
    let normalized_tokens = normalize_tokens(tokens, stemmer);
    let normalized_tokens_ref = normalized_tokens.iter().map(|t| &**t).collect_vec();
    let mut filtered_ngrams = compute_all_ngrams(&*normalized_tokens_ref, normalized_tokens_ref.len())
        .into_iter()
        .filter(|ngram_indexes| ngram_indexes.1.iter().any(|index| *index == token_index))
        .collect_vec();

    filtered_ngrams.sort_by_key(|ngrams| -(ngrams.1.len() as i64));

    filtered_ngrams.iter()
        .find(|ngrams| gazetteer.contains(&ngrams.0))
        .map(|ngrams| get_scheme_prefix(token_index, &ngrams.1, tagging_scheme).to_string())
}

pub fn get_word_cluster<C: WordClusterer, S: Stemmer>(tokens: &[Token],
                                                      token_index: usize,
                                                      word_clusterer: &C,
                                                      stemmer: Option<&S>) -> Option<String> {
    if token_index >= tokens.len() {
        return None;
    }
    let normalized_token = if let Some(stemmer) = stemmer {
        stemmer.stem(&tokens[token_index].value.to_lowercase())
    } else {
        tokens[token_index].value.to_lowercase()
    };
    word_clusterer.get_cluster(&normalized_token)
}

pub fn get_builtin_entities_annotation(tokens: &[Token],
                                       token_index: usize,
                                       parser: &RustlingParser,
                                       builtin_entity_kind: EntityKind,
                                       tagging_scheme: TaggingScheme) -> Option<String> {
    if token_index >= tokens.len() {
        return None;
    }
    let text = initial_string_from_tokens(tokens);
    parser.extract_entities(&text, Some(&[builtin_entity_kind]))
        .into_iter()
        .find(|e| ranges_overlap(&e.char_range, &tokens[token_index].char_range))
        .map(|e| {
            let entity_token_indexes = (0..tokens.len())
                .filter(|i| ranges_overlap(&tokens[*i].char_range, &e.char_range))
                .collect_vec();
            get_scheme_prefix(token_index, &entity_token_indexes, tagging_scheme).to_string()
        })
}

fn normalize_tokens<S: Stemmer>(tokens: &[Token], stemmer: Option<&S>) -> Vec<String> {
    tokens.iter()
        .map(|t|
            stemmer.map_or(t.value.to_lowercase(), |s| s.stem(&t.value.to_lowercase()))
        ).collect_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    use utils::token::tokenize;
    use rustling_ontology::Lang;

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
        let tokens = tokenize("Hello BEAUTIFUL world !!!");

        // When
        let actual_result = vec![shape(&tokens, 0, 2), shape(&tokens, 1, 3)];

        // Then
        let expected_result = vec![Some("Xxx XXX".to_string()), Some("XXX xxx xX".to_string())];
        assert_eq!(actual_result, expected_result)
    }

    fn assert_ngrams_eq<S: Stemmer, G: Gazetteer>(expected_ngrams: Vec<Vec<Option<String>>>,
                                                  tokens: &[Token],
                                                  stemmer: Option<&S>,
                                                  gazetteer: Option<&G>) {
        for (n, expected_ngrams) in expected_ngrams.iter().enumerate() {
            for (i, expected_ngram) in expected_ngrams.iter().enumerate() {
                let actual_ngrams = ngram(tokens, i, n + 1, stemmer, gazetteer);
                assert_eq!(*expected_ngram, actual_ngrams)
            }
        }
    }

    #[test]
    fn ngram_works() {
        let tokens = tokenize("I love House Music");

        let expected_ngrams = vec![vec![Some("i".to_string()),
                                        Some("love".to_string()),
                                        Some("house".to_string()),
                                        Some("music".to_string())],
                                   vec![Some("i love".to_string()),
                                        Some("love house".to_string()),
                                        Some("house music".to_string()),
                                        None],
                                   vec![Some("i love house".to_string()),
                                        Some("love house music".to_string()),
                                        None,
                                        None]];

        assert_ngrams_eq(expected_ngrams,
                         &tokens,
                         None as Option<&StaticMapStemmer>,
                         None as Option<&HashSetGazetteer>);
    }


    #[test]
    fn ngram_works_with_common_words_gazetteer() {
        // Given
        let tokens = tokenize("I love House Music");
        let common_words_gazetteer = HashSetGazetteer::from(
            vec![
                "i".to_string(),
                "love".to_string(),
                "music".to_string()
            ].into_iter()
        );

        // Then
        let expected_ngrams = vec![vec![Some("i".to_string()),
                                        Some("love".to_string()),
                                        Some("rare_word".to_string()),
                                        Some("music".to_string())],
                                   vec![Some("i love".to_string()),
                                        Some("love rare_word".to_string()),
                                        Some("rare_word music".to_string()),
                                        None],
                                   vec![Some("i love rare_word".to_string()),
                                        Some("love rare_word music".to_string()),
                                        None,
                                        None]];

        assert_ngrams_eq(expected_ngrams,
                         &tokens,
                         None as Option<&StaticMapStemmer>,
                         Some(&common_words_gazetteer));
    }

    #[test]
    fn ngram_works_with_stemmer() {
        // Given
        let tokens = tokenize("I love House Music");
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
        let expected_ngrams = vec![vec![Some("i".to_string()),
                                        Some("love".to_string()),
                                        Some("hous".to_string()),
                                        Some("music".to_string())],
                                   vec![Some("i love".to_string()),
                                        Some("love hous".to_string()),
                                        Some("hous music".to_string()),
                                        None],
                                   vec![Some("i love hous".to_string()),
                                        Some("love hous music".to_string()),
                                        None,
                                        None]];

        assert_ngrams_eq(expected_ngrams,
                         &tokens,
                         Some(&stemmer),
                         None as Option<&HashSetGazetteer>);
    }

    #[test]
    fn is_in_gazetteer_works() {
        // Given
        let gazetteer = HashSetGazetteer::from(
            vec![
                "bird".to_string(),
                "blue bird".to_string(),
                "beautiful blue bird".to_string()
            ].into_iter()
        );
        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love this beautiful blue Bird !");
        let token_index = 5;

        // When
        let actual_result = is_in_gazetteer(
            &tokens,
            token_index,
            &gazetteer,
            None as Option<&StaticMapStemmer>,
            tagging_scheme);

        // Then
        assert_eq!(Some("L-".to_string()), actual_result)
    }

    #[test]
    fn is_in_gazetteer_works_with_stemming() {
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

        let stemmer = TestStemmer {};
        let gazetteer = HashSetGazetteer::from(
            vec![
                "bird".to_string(),
                "blue bird".to_string(),
                "beautiful blue bird".to_string()
            ].into_iter()
        );

        let tagging_scheme = TaggingScheme::BILOU;
        let tokens = tokenize("I love Blue Birds !");
        let token_index = 3;

        // When
        let actual_result = is_in_gazetteer(
            &tokens,
            token_index,
            &gazetteer,
            Some(&stemmer),
            tagging_scheme);

        // Then
        assert_eq!(Some("L-".to_string()), actual_result)
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

        let word_clusterer = TestWordClusterer {};
        let tokens = tokenize("I love this bird");
        let token_index = 3;

        // When
        let actual_result = get_word_cluster(&tokens, token_index, &word_clusterer, None as Option<&StaticMapStemmer>);

        // Then
        assert_eq!(Some("010101".to_string()), actual_result);
    }

    #[test]
    fn get_builtin_annotation_works() {
        // Given
        let tokens = tokenize("Let's meet tomorrow at 9pm ok ?");
        let token_index = 5; // 9pm
        let tagging_scheme = TaggingScheme::BILOU;
        let parser = RustlingParser::get(Lang::EN);

        // When
        let actual_annotation = get_builtin_entities_annotation(&tokens,
                                                                token_index,
                                                                &*parser,
                                                                EntityKind::Time,
                                                                tagging_scheme);

        // Then
        assert_eq!(Some("L-".to_string()), actual_annotation)
    }
}
