mod normalization;
mod entity;
mod tokenization;
mod preprocessor_result;
pub use self::preprocessor_result::PreprocessorResult;

use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    value: String,
    range: Range<usize>,
    // TODO is this really needed? if only used in tests, remove
    char_range: Range<usize>,
    entity: Option<String>,
}

impl Token {
    fn to_normalized(&self) -> NormalizedToken {
        NormalizedToken {
            value: self.value.clone(),
            normalized_value: normalization::normalize(&self.value),
            range: self.range.clone(),
            char_range: self.char_range.clone(),
            entity: self.entity.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NormalizedToken {
    pub value: String,
    pub normalized_value: String,
    pub range: Range<usize>,
    pub char_range: Range<usize>,
    pub entity: Option<String>,
}

pub fn preprocess(input: &str) -> PreprocessorResult {
    let normalized_tokens = tokenization::tokenize(entity::detect_entities(input))
        .iter()
        .map(|token| token.to_normalized())
        .collect();

    PreprocessorResult::new(normalized_tokens)
}

fn convert_char_index(string: &str, byte_index: usize) -> usize {
    if string.is_empty() {
        return 0;
    }
    let mut acc = 0;
    let mut last_char_index = 0;
    for (char_index, char) in string.chars().enumerate() {
        if byte_index <= acc {
            return char_index;
        }
        acc += char.len_utf8();
        last_char_index = char_index;
    }
    last_char_index + 1
}


#[cfg(test)]
pub fn convert_byte_index(string: &str, char_index: usize) -> usize {
    let mut result = 0;
    for (current_char_index, char) in string.chars().enumerate() {
        if current_char_index == char_index {
            return result;
        }
        result += char.len_utf8()
    }
    result
}
