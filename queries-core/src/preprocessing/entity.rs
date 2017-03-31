use std::ops::Range;
use std::iter::Iterator;

use preprocessing::Token;
use preprocessing::convert_char_index;

pub fn detect_entities(input: &str, entity_tokens: Vec<Token>) -> Vec<Token> {
    let entity_ranges: Vec<Range<usize>> =
        entity_tokens.iter().map(|token| token.range.clone()).collect();

    #[derive(Debug, PartialEq)]
    struct Index {
        byte: usize,
        char: usize,
    }

    let mut indices: Vec<Index> = entity_tokens.iter()
        .flat_map(|token| {
            vec![Index {
                     byte: token.range.start,
                     char: token.char_range.start,
                 },
                 Index {
                     byte: token.range.end,
                     char: token.char_range.end,
                 }]
        })
        .chain(vec![Index { byte: 0, char: 0 },
                    Index {
                        byte: input.len(),
                        char: convert_char_index(input, input.len()),
                    }])
        .collect();

    indices.sort_by_key(|index| index.byte);
    indices.dedup();

    let mut result: Vec<Token> = indices.iter()
        .zip(indices[1..indices.len()].iter())
        .map(|(start, end)| {
            Token {
                value: unsafe { input.slice_unchecked(start.byte, end.byte).to_string() },
                entity: None,
                char_range: Range {
                    start: start.char,
                    end: end.char,
                },
                range: Range {
                    start: start.byte,
                    end: end.byte,
                },
            }
        })
        .filter(|tested_token| !entity_ranges.iter().any(|range| *range == tested_token.range))
        .chain(entity_tokens)
        .collect();

    result.sort_by_key(|token| token.range.start);

    result
}
