use std::ascii::AsciiExt;
use itertools::Itertools;
use preprocessing::Token;


pub fn is_first(i: usize) -> Option<String> {
    if i == 0 { Some("1".to_string()) } else { None }
}

pub fn is_last(t: &[Token], i: usize) -> Option<String> {
    if i == t.len() - 1 { Some("1".to_string()) } else { None }
}

// TODO add stemization & gazetteer support
pub fn ngram(t: &[Token], i: usize, n: usize) -> Option<String> {
    // TODO we should precompute the ascii lowercase value somewhere, perhaps use NormalizedToken ?
    if i + n > t.len() { None } else {
        Some(t[i..i + n].iter().map(|t| t.value.to_ascii_lowercase()).join(" "))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use preprocessing::light::tokenize;

    #[test]
    fn ngram_works() {
        let tokens = tokenize("I love house music");

        let expected_ngrams = vec![
            vec![Some("i".to_string()),
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
                 None]
        ];

        for (n, expected_ngrams) in expected_ngrams.iter().enumerate() {
            for (i, expected_ngram) in expected_ngrams.iter().enumerate() {
                assert_eq!(ngram(&tokens, i, n + 1), *expected_ngram)
            }
        }
    }
}
