use std::ops::Range;

use errors::*;
use serde_json;

mod normalization;
mod entity;
mod tokenization;
mod preprocessor_result;
pub use self::preprocessor_result::PreprocessorResult;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct EntityToken {
    value: String,
    start_index: usize,
    end_index: usize,
    entity: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    value: String,
    range: Range<usize>,
    // TODO is this really needed? if only used in tests, remove
    char_range: Range<usize>,
    entity: Option<String>,
}

impl Token {
    fn from(input: &str, entity_token: &EntityToken) -> Token {
        Token {
            range: Range {
                start: convert_byte_index(input, entity_token.start_index),
                end: convert_byte_index(input, entity_token.end_index),
            },
            char_range: Range {
                start: entity_token.start_index,
                end: entity_token.end_index,
            },
            value: entity_token.value.clone(),
            entity: Some(entity_token.entity.clone()),
        }
    }

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

pub fn create_normalized_tokens(input: &str, entity_tokens: Vec<Token>) -> Vec<NormalizedToken> {
    tokenization::tokenize(entity::detect_entities(input, entity_tokens))
        .iter()
        .map(|token| token.to_normalized())
        .collect()
}

pub fn preprocess(input: &str, entities: &str) -> Result<PreprocessorResult> {
    let entity_tokens = serde_json::from_str::<Vec<EntityToken>>(entities)?
        .iter()
        .map(|entity_token| Token::from(input, entity_token))
        .collect();

    Ok(PreprocessorResult::new(input.into(), create_normalized_tokens(input, entity_tokens)))
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
