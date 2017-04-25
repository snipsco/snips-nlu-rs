use std::ops::Range;

use rustling_ontology::*;

use errors::*;

mod normalization;
mod entity;
mod tokenization;
mod preprocessor_result;
pub use self::preprocessor_result::PreprocessorResult;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    value: String,
    range: Range<usize>,
    // TODO is this really needed? if only used in tests, remove
    char_range: Range<usize>,
    entity: Option<String>,
}

trait EntityMapping {
    fn to_entity(&self) -> Option<&'static str>;
}

impl EntityMapping for Dimension {
    fn to_entity(&self) -> Option<&'static str> {
        match self.kind() {
            DimensionKind::Number => Some("%NUMBER%"),
            DimensionKind::Ordinal => Some("%ORDINAL%"),
            DimensionKind::AmountOfMoney => Some("%PRICE%"),
            DimensionKind::Temperature => Some("%TEMPERATURE%"),
            _ => None
        }
    }
}

impl Token {
    fn from(input: &str, parser_match: ParserMatch<Dimension>) -> Token {
        let range = Range {
            start: parser_match.range.0,
            end: parser_match.range.1,
        };

        Token {
            range: range.clone(),
            char_range: Range {
                start: convert_char_index(input, range.start),
                end: convert_char_index(input, range.end),
            },
            value: input[range].to_string(),
            entity: parser_match.value.to_entity().map(|e| e.to_string()),
        }
    }

    fn to_normalized(self) -> NormalizedToken {
        NormalizedToken {
            normalized_value: normalization::normalize(&self.value),
            value: self.value,
            range: self.range,
            char_range: self.char_range,
            entity: self.entity,
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
        .into_iter()
        .map(|token| token.to_normalized())
        .collect()
}

pub fn preprocess(input: &str) -> Result<PreprocessorResult> {
    let parser = build_parser(Lang::EN)?;
    let entities = parser.parse(input)?;

    let entity_tokens = entities.into_iter().map(|pm| Token::from(input, pm)).collect();

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
