use std::ascii::AsciiExt;
use itertools::Itertools;
use preprocessing::Token;


pub fn is_first(i: usize) -> Option<(String, String)> {
    if i == 0 { Some(("is_first".to_string(), "1".to_string())) } else { None }
}

pub fn is_last(t: &[Token], i: usize) -> Option<(String, String)> {
    if i == t.len() - 1 { Some(("is_last".to_string(), "1".to_string())) } else { None }
}

pub fn ngram(t: &[Token], i: usize, n: usize) -> Option<(String, String)> {
    // TODO we should precompute the ascii lowercase value somewhere, perhaps use NormalizedToken ?
    if i + n > t.len() { None } else {
        Some((format!("ngram_{}", n),
              t[i..i + n].iter().map(|t| t.value.to_ascii_lowercase()).join(" ")))
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
            vec![Some(("ngram_1".to_string(), "i".to_string())),
                 Some(("ngram_1".to_string(), "love".to_string())),
                 Some(("ngram_1".to_string(), "house".to_string())),
                 Some(("ngram_1".to_string(), "music".to_string()))],
            vec![Some(("ngram_2".to_string(), "i love".to_string())),
                 Some(("ngram_2".to_string(), "love house".to_string())),
                 Some(("ngram_2".to_string(), "house music".to_string())),
                 None],
            vec![Some(("ngram_3".to_string(), "i love house".to_string())),
                 Some(("ngram_3".to_string(), "love house music".to_string())),
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
