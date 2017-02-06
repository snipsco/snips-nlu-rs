use std::ops::Range;
use preprocessing::Token;
use preprocessing::convert_char_index;
use regex::Regex;


pub fn tokenize(input: Vec<Token>) -> Vec<Token> {
    input.iter()
        .flat_map(extract_punctuation_on_simple_token)
        .flat_map(split_spaces_on_simple_token)
        .collect()
}

fn extract_punctuation(token: &Token) -> Vec<Token> {
    lazy_static! {
        static ref PUNCTUATION_REGEX: Regex = Regex::new(r"[.,;:?!…\n\r]+").unwrap();
    }
    let mut indexes = PUNCTUATION_REGEX.find_iter(&token.value)
        .flat_map(|find| vec![find.0, find.1])
        .collect::<Vec<usize>>();

    indexes.push(0);
    indexes.push(token.value.len());
    indexes.sort();
    indexes.dedup();
    indexes.iter()
        .zip(indexes[1..indexes.len()].iter())
        .map(|(start, end)| {
            Token {
                value: unsafe { token.value.slice_unchecked(*start, *end).to_string() },
                entity: token.entity.clone(),
                char_range: Range {
                    start: token.char_range.start + convert_char_index(&token.value, *start),
                    end: token.char_range.start + convert_char_index(&token.value, *end),
                },
                range: Range {
                    start: token.range.start + start,
                    end: token.range.start + end,
                },
            }
        })
        .collect()
}

fn extract_punctuation_on_simple_token(token: &Token) -> Vec<Token> {
    do_on_simple_token(token, extract_punctuation)
}

fn split_spaces(token: &Token) -> Vec<Token> {
    lazy_static! {
        static ref SPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
    }

    let mut result: Vec<Token> = Vec::new();

    let mut start_index = token.range.start;
    let mut char_start_index = token.char_range.start;

    for item in SPACE_REGEX.split(&token.value) {
        let end_index = start_index + item.len();
        let char_end_index = char_start_index + convert_char_index(item, item.len());
        if !item.is_empty() {
            result.push(Token {
                value: item.to_string(),
                entity: token.entity.clone(),
                char_range: Range {
                    start: char_start_index,
                    end: char_end_index,
                },
                range: Range {
                    start: start_index,
                    end: end_index,
                },
            });
        }
        start_index = end_index + 1;
        char_start_index = char_end_index + 1;
    }

    result
}

fn split_spaces_on_simple_token(token: Token) -> Vec<Token> {
    do_on_simple_token(&token, split_spaces)
}

fn do_on_simple_token(token: &Token, f: fn(&Token) -> Vec<Token>) -> Vec<Token> {
    match token.entity {
        Some(_) => vec![token.clone()],
        None => f(token),
    }
}

#[cfg(test)]
mod test {
    use std::ops::Range;
    use preprocessing::Token;
    use preprocessing::convert_byte_index;
    use testutils::parse_json;
    use super::tokenize;

    #[derive(Deserialize, Debug)]
    struct TestDescription {
        description: String,
        base_input: String,
        input: Vec<TestToken>,
        output: Vec<TestToken>,
    }

    #[derive(Deserialize, Debug)]
    struct TestToken {
        value: String,
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "endIndex")]
        end_index: usize,
        entity: Option<String>,
    }

    impl TestToken {
        fn to_token(&self, base_string: &str) -> Token {
            Token {
                value: self.value.clone(),
                entity: self.entity.clone(),
                char_range: Range {
                    start: self.start_index,
                    end: self.end_index,
                },
                range: Range {
                    start: convert_byte_index(base_string, self.start_index),
                    end: convert_byte_index(base_string, self.end_index),
                },
            }
        }
    }

    #[test]
    fn tokenization_works() {
        let tests: Vec<TestDescription> = parse_json("snips-sdk-tests/preprocessing/tokenization/span_tokenization.json");
        assert!(tests.len() != 0);
        for test in tests {
            let result =
                tokenize(test.input.iter().map(|x| x.to_token(&test.base_input)).collect());
            for (index, expected_token) in
                test.output.iter().map(|x| x.to_token(&test.base_input)).enumerate() {
                assert_eq!(result[index], expected_token);
            }
        }
    }
}
