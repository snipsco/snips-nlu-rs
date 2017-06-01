use std::str;
use std::iter::FromIterator;
use regex::{Regex, RegexBuilder};
use utils::token::Token;

pub fn get_word_chunk(word: String, chunk_size: usize, chunk_start: usize, reverse: bool) -> Option<String> {
    if reverse && chunk_size > chunk_start {
        return None;
    }
    let start = if reverse { chunk_start - chunk_size } else { chunk_start };
    if start + chunk_size > word.chars().count() {
        None
    } else {
        Some(word.chars().skip(start).take(chunk_size).collect())
    }
}

pub fn get_shape(string: &str) -> String {
    lazy_static! {
        static ref LOWER_REGEX: Regex = RegexBuilder::new("^[a-z]+$").case_insensitive(false).build().unwrap();
        static ref UPPER_REGEX: Regex = RegexBuilder::new("^[A-Z]+$").case_insensitive(false).build().unwrap();
        static ref TITLE_REGEX: Regex = RegexBuilder::new("^[A-Z][a-z]+$").case_insensitive(false).build().unwrap();
    }

    if LOWER_REGEX.is_match(string) {
        "xxx".to_string()
    } else if UPPER_REGEX.is_match(string) {
        "XXX".to_string()
    } else if TITLE_REGEX.is_match(string) {
        "Xxx".to_string()
    } else {
        "xX".to_string()
    }
}

pub fn initial_string_from_tokens(tokens: &[Token]) -> String {
    let mut current_index = 0;
    let mut chunks: Vec<String> = Vec::with_capacity(2 * tokens.len() - 1);
    for token in tokens {
        if token.char_range.start > current_index {
            let nb_spaces = token.char_range.start - current_index;
            let spaces = String::from_iter(vec![' '; nb_spaces].into_iter());
            chunks.push(spaces);
        }
        chunks.push(token.value.clone());
        current_index = token.char_range.end;
    }
    chunks.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_word_chunk_works() {
        // Given
        let word = "hello_world".to_string();
        let chunk_size = 6;
        let chunk_start = 5;
        let reverse = false;

        // When
        let word_chunk = get_word_chunk(word, chunk_size, chunk_start, reverse);

        // Then
        let expected_chunk = Some("_world".to_string());
        assert_eq!(word_chunk, expected_chunk);
    }

    #[test]
    fn get_word_chunk_reversed_works() {
        // Given
        let word = "hello_world".to_string();
        let chunk_size = 8;
        let chunk_start = 8;
        let reverse = true;

        // When
        let word_chunk = get_word_chunk(word, chunk_size, chunk_start, reverse);

        // Then
        let expected_chunk = Some("hello_wo".to_string());
        assert_eq!(word_chunk, expected_chunk);
    }

    #[test]
    fn get_word_chunk_out_of_bound_works() {
        // Given
        let word = "hello_world".to_string();
        let chunk_size = 4;
        let chunk_start = 8;
        let reverse = false;

        // When
        let word_chunk = get_word_chunk(word, chunk_size, chunk_start, reverse);

        // Then
        assert_eq!(word_chunk, None);
    }

    #[test]
    fn get_shape_works() {
        // Given
        let inputs = vec!["hello", "Hello", "HELLO", "heLo", "!!"];

        // When
        let actual_shapes: Vec<String> = (0..5).map(|i| get_shape(inputs[i])).collect();

        // Then
        let expected_shapes = vec!["xxx", "Xxx", "XXX", "xX", "xX"];
        assert_eq!(actual_shapes, expected_shapes)
    }

    #[test]
    fn initial_string_from_tokens_works() {
        // Given
        let tokens = vec![
            Token {
                value: "hello".to_string(),
                range: 0..5,
                char_range: 0..5,
            },
            Token {
                value: "world".to_string(),
                range: 9..14,
                char_range: 9..14,
            },
            Token {
                value: "!!!".to_string(),
                range: 17..20,
                char_range: 17..20,
            }
        ];

        // When
        let result = initial_string_from_tokens(&tokens);

        // Then
        assert_eq!("hello    world   !!!", &result);
    }
}
