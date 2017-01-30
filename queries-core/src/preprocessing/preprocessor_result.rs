use std::cmp::min;
use preprocessing::NormalizedToken;
use itertools::Itertools;

type Ngrams = (String, Vec<usize>);

pub struct PreprocessorResult {
    pub ngrams: Vec<Ngrams>,
    pub normalized_ngrams: Vec<Ngrams>,
    pub formatted_ngrams: Vec<Ngrams>,
}

impl PreprocessorResult {
    pub fn new(tokens: &Vec<NormalizedToken>) -> PreprocessorResult {
        return PreprocessorResult {
            ngrams: compute_all_ngrams(&*tokens.iter().map(|token| &*token.value).collect_vec(),
                                       tokens.len()),
            normalized_ngrams: compute_all_ngrams(&*tokens.iter()
                                                      .map(|token| &*token.normalized_value)
                                                      .collect_vec(),
                                                  tokens.len()),
            formatted_ngrams: compute_all_ngrams(&*tokens.iter()
                                                     .map(|token| {
                                                         &**token.entity
                                                             .as_ref()
                                                             .unwrap_or(&token.normalized_value)
                                                     })
                                                     .collect_vec(),
                                                 tokens.len()),
        };
    }
}

// TODO: This func should be optimized
fn compute_all_ngrams(tokens: &[&str], max_ngram_size: usize) -> Vec<Ngrams> {
    let mut ngrams: Vec<Ngrams> = Vec::new();

    for start in 0..tokens.len() {
        let mut local_ngrams: Vec<Ngrams> = Vec::new();
        let mut last_ngram_item: Option<Ngrams> = None;
        let max_end = min(tokens.len(), start + max_ngram_size);

        for end in start..max_end {
            let ngram_item = if let Some(last_ngram_item) = last_ngram_item {
                (format!("{} {}", last_ngram_item.0, tokens[end]),
                 consume_and_concat(last_ngram_item.1, vec![end]))
            } else {
                (tokens[start].to_string(), vec![start])
            };
            last_ngram_item = Some(ngram_item.clone());
            local_ngrams.push(ngram_item);
        }
        ngrams.extend_from_slice(&local_ngrams);
    }

    ngrams
}

fn consume_and_concat<T>(mut vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    vec1.extend(vec2);
    vec1
}

#[cfg(test)]
mod test {
    use super::Ngrams;
    use super::compute_all_ngrams;

    #[test]
    fn compute_all_ngrams_works() {
        let result = compute_all_ngrams(&vec!["a", "b", "c"], 3);
        let expected: Vec<Ngrams> = vec![("a".to_string(), vec![0]),
                                         ("a b".to_string(), vec![0, 1]),
                                         ("a b c".to_string(), vec![0, 1, 2]),
                                         ("b".to_string(), vec![1]),
                                         ("b c".to_string(), vec![1, 2]),
                                         ("c".to_string(), vec![2])];
        assert_eq!(result, expected)
    }
}
