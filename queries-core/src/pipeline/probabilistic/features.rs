use std::ascii::AsciiExt;
use itertools::Itertools;
use preprocessing::Token;
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
pub fn ngram(t: &[Token], i: usize, n: usize) -> Option<String> {
    // TODO we should precompute the ascii lowercase value somewhere, perhaps use NormalizedToken ?
    if i + n > t.len() {
        None
    } else {
        Some(t[i..i + n].iter().map(|t| t.value.to_ascii_lowercase()).join(" "))
    }
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
}
