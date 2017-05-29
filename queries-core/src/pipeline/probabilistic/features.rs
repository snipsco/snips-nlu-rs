use itertools::Itertools;
use preprocessing::Token;
use preprocessing::compute_all_ngrams;
use models::gazetteer::Gazetteer;
use models::stemmer::Stemmer;
use resources_packed::word_cluster;
use super::crf_utils::{TaggingScheme, get_scheme_prefix};
use super::features_utils::{get_word_chunk, get_shape};


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


// TODO add stemization & gazetteer support
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
                                                 tagging_scheme: &TaggingScheme) -> Option<String> {
    let normalized_tokens = normalize_tokens(tokens, stemmer);
    let normalized_tokens_ref = normalized_tokens.iter().map(|t| &**t).collect_vec();
    let mut filtered_ngrams = compute_all_ngrams(&*normalized_tokens_ref, normalized_tokens_ref.len())
        .into_iter()
        .filter(|ngram_indexes| ngram_indexes.1.iter().any(|index| *index == token_index))
        .collect_vec();

    filtered_ngrams.sort_by_key(|ngrams| -(ngrams.1.len() as i64));

    filtered_ngrams.iter()
        .find(|ngrams| gazetteer.contains(&ngrams.0))
        .map(|ngrams| get_scheme_prefix(token_index, &ngrams.1, tagging_scheme))
}

pub fn get_word_cluster(tokens: &[Token],
                        token_index: usize,
                        cluster_name: &str,
                        language_code: &str) -> Option<String> {
    if token_index >= tokens.len() {
        None
    } else {
        word_cluster(cluster_name, language_code, &tokens[token_index].value.to_lowercase()).unwrap()
    }
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

    use preprocessing::tokenize;

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

    #[test]
    fn ngram_works() {
        let tokens = tokenize("I love house music");

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

        for (n, expected_ngrams) in expected_ngrams.iter().enumerate() {
            for (i, expected_ngram) in expected_ngrams.iter().enumerate() {
                assert_eq!(ngram(&tokens, i, n + 1), *expected_ngram)
            }
        }
    }

    #[test]
    fn is_in_collection_works() {
        // Given
        let tokens = tokenize("I love this beautiful blue bird !");
        let collection = vec![
            "bird".to_string(),
            "blue bird".to_string(),
            "beautiful blue bird".to_string()
        ];
        let tagging_scheme = TaggingScheme::BIO;

        // When
        let actual_results = vec![
            is_in_collection(&tokens, 2, &collection, &tagging_scheme),
            is_in_collection(&tokens, 3, &collection, &tagging_scheme),
            is_in_collection(&tokens, 4, &collection, &tagging_scheme)
        ];

        // Then
        let expected_results = vec![
            None,
            Some("B-".to_string()),
            Some("I-".to_string())
        ];

        assert_eq!(actual_results, expected_results);
    }
}
