use std::iter::FromIterator;
use std::str;

use nlu_utils::token::Token;

pub fn get_word_chunk(
    word: &str,
    chunk_size: usize,
    chunk_start: usize,
    reverse: bool,
) -> Option<String> {
    if reverse && chunk_size > chunk_start {
        return None;
    }
    let start = if reverse {
        chunk_start - chunk_size
    } else {
        chunk_start
    };
    if start + chunk_size > word.chars().count() {
        None
    } else {
        Some(word.chars().skip(start).take(chunk_size).collect())
    }
}

pub fn initial_string_from_tokens(tokens: &[Token]) -> String {
    let mut current_index = 0;
    let mut chunks: Vec<String> = Vec::with_capacity(2 * tokens.len() - 1);
    for token in tokens {
        if token.char_range.start > current_index {
            let nb_spaces = token.char_range.start - current_index;
            let spaces = String::from_iter(vec![' '; nb_spaces]);
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
        let word = "hello_world";
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
        let word = "hello_world";
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
        let word = "hello_world";
        let chunk_size = 4;
        let chunk_start = 8;
        let reverse = false;

        // When
        let word_chunk = get_word_chunk(word, chunk_size, chunk_start, reverse);

        // Then
        assert_eq!(word_chunk, None);
    }

    #[test]
    fn initial_string_from_tokens_works() {
        // Given
        let tokens = vec![
            Token::new("hello".to_string(), 0..5, 0..5),
            Token::new("world".to_string(), 9..14, 9..14),
            Token::new("!!!".to_string(), 17..20, 17..20),
        ];

        // When
        let result = initial_string_from_tokens(&tokens);

        // Then
        assert_eq!("hello    world   !!!", &result);
    }
}
