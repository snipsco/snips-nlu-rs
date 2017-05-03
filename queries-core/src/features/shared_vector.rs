use regex::Regex;
use yolo::Yolo;

use preprocessing::PreprocessorResult;
use models::gazetteer::Gazetteer;

pub fn has_gazetteer_hits(preprocessor_result: &PreprocessorResult,
                          gazetteer: Box<Gazetteer>)
                          -> Vec<f32> {
    let mut result = vec![0.0; preprocessor_result.tokens.len()];

    for ref ngram in &preprocessor_result.normalized_ngrams {
        if gazetteer.contains(&ngram.0) {
            for index in &ngram.1 {
                result[*index as usize] = 1.0;
            }
        }
    }
    result
}

pub fn ngram_matcher(preprocessor_result: &PreprocessorResult, ngram_to_check: &str) -> Vec<f32> {
    let mut result = vec![0.0; preprocessor_result.tokens.len()];

    for ref ngram in &preprocessor_result.formatted_ngrams {
        if &ngram.0 == ngram_to_check {
            for index in &ngram.1 {
                result[*index as usize] = 1.0;
            }
        }
    }
    result
}

pub fn is_capitalized(preprocessor_result: &PreprocessorResult) -> Vec<f32> {
    preprocessor_result.tokens
        .iter()
        .map(|token| if let Some(first_char) = token.value.chars().next() {
            if first_char.is_uppercase() { 1.0 } else { 0.0 }
        } else {
            0.0
        })
        .collect()
}

pub fn is_first_word(preprocessor_result: &PreprocessorResult) -> Vec<f32> {
    lazy_static! {
        static ref PUNCTUATIONS: Vec<&'static str> = vec![",", ".", "?"];
    }

    let ref tokens = preprocessor_result.tokens;
    let tokens_count = tokens.len();
    let mut result = vec![0.0; tokens_count];

    let mut i = 0;
    while i < tokens_count && PUNCTUATIONS.contains(&&*tokens[i].normalized_value) {
        i = i + 1;
    }
    if i < tokens_count {
        result[i] = 1.0;
    }
    result
}

pub fn is_last_word(preprocessor_result: &PreprocessorResult) -> Vec<f32> {
    lazy_static! {
        static ref PUNCTUATIONS: Vec<&'static str> = vec![",", ".", "?"];
    }

    let ref tokens = preprocessor_result.tokens;
    let tokens_count = tokens.len();
    let mut result = vec![0.0; tokens_count];

    let mut i = tokens_count - 1;
    while PUNCTUATIONS.contains(&&*tokens[i].normalized_value) {
        i = i - 1;
    }
    result[i] = 1.0;
    result
}

pub fn contains_possessive(preprocessor_result: &PreprocessorResult) -> Vec<f32> {
    lazy_static! {
        static ref POSSESSIVE_REGEX: Regex = Regex::new(r"'s\b").yolo();
    }

    let ref tokens = preprocessor_result.tokens;
    tokens.iter()
        .map(|t| if POSSESSIVE_REGEX.is_match(&t.normalized_value) {
            1.0
        } else {
            0.0
        })
        .collect()
}

pub fn contains_digits(preprocessor_result: &PreprocessorResult) -> Vec<f32> {
    lazy_static! {
        static ref DIGITS_REGEX: Regex = Regex::new(r"[0-9]").yolo();
    }

    let ref tokens = preprocessor_result.tokens;
    tokens.iter()
        .map(|t| if DIGITS_REGEX.is_match(&t.normalized_value) {
            1.0
        } else {
            0.0
        })
        .collect()
}

#[cfg(test)]
mod test {
    use std::path;

    use serde_json;

    use preprocessing::{NormalizedToken, PreprocessorResult};
    use models::gazetteer::HashSetGazetteer;
    use utils::parse_json;
    use utils::convert_byte_range;

    use super::has_gazetteer_hits;
    use super::ngram_matcher;
    use super::is_capitalized;
    use super::is_first_word;
    use super::is_last_word;
    use super::contains_possessive;

    #[derive(Debug, PartialEq, Clone, Deserialize)]
    struct TestDescription {
        //description: String,
        input: Input,
        args: Vec<Arg>,
        output: Vec<f32>,
    }

    #[derive(Debug, PartialEq, Clone, Deserialize)]
    struct Input {
        text: String,
        tokens: Vec<Token>,
    }

    #[derive(Debug, PartialEq, Clone, Deserialize)]
    struct Token {
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "endIndex")]
        end_index: usize,
        normalized: String,
        value: String,
        entity: Option<String>,
    }

    #[derive(Debug, PartialEq, Clone, Deserialize)]
    struct Arg {
        //#[serde(rename = "type")]
        //kind: String,
        //name: String,
        value: Data,
    }

    #[derive(Debug, PartialEq, Clone, Deserialize)]
    #[serde(untagged)]
    enum Data {
        StringValue(String),
        StringArray(Vec<String>),
        Float(f32),
    }

    impl Token {
        fn to_normalized_token(self, base_string: &str) -> NormalizedToken {
            let range = self.start_index..self.end_index;
            NormalizedToken {
                value: self.value,
                normalized_value: self.normalized,
                range: convert_byte_range(base_string, &range),
                char_range: range,
                entity: self.entity,
            }
        }
    }

    #[test]
    fn feature_function_works() {
        let tests: Vec<(&str, Box<Fn(&TestDescription, &PreprocessorResult)>)> = vec![
            ("hasGazetteerHits", Box::new(has_gazetteer_hits_works)),
            ("ngramMatcher", Box::new(ngram_matcher_works)),
            ("isCapitalized", Box::new(is_capitalized_works)),
            ("isFirstWord", Box::new(is_first_word_works)),
            ("isLastWord", Box::new(is_last_word_works)),
            ("containsPossessive", Box::new(contains_possessive_works)),
        ];

        let path = path::PathBuf::from("snips-sdk-tests/feature_extraction/SharedVector");

        for test in tests {
            let test_name = test.0;
            let test_path = path.join(&test_name).with_extension("json");
            let parsed_tests: Vec<TestDescription> = parse_json(&test_path.to_str().unwrap());
            assert!(parsed_tests.len() != 0);

            for parsed_test in parsed_tests {
                let normalized_tokens: Vec<NormalizedToken> = parsed_test.input.tokens
                    .clone()
                    .into_iter()
                    .map(|test_token| test_token.to_normalized_token(&parsed_test.input.text))
                    .collect();

                let preprocessor_result = PreprocessorResult::new(parsed_test.input.text.to_string(), normalized_tokens);
                test.1(&parsed_test, &preprocessor_result);
            }
        }
    }

    fn has_gazetteer_hits_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let values = if let Data::StringArray(ref v) = test.args[0].value { v } else { panic!() };

        let gazetteer = HashSetGazetteer::new(&mut serde_json::to_string(values).unwrap().as_bytes()).unwrap();

        let result = has_gazetteer_hits(&preprocessor_result, Box::new(gazetteer));
        assert_eq!(result, test.output)
    }

    fn ngram_matcher_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let ngram = if let Data::StringValue(ref v) = test.args[0].value { v } else { panic!() };
        let result = ngram_matcher(&preprocessor_result, &ngram);
        assert_eq!(result, test.output)
    }

    fn is_capitalized_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let result = is_capitalized(&preprocessor_result);
        assert_eq!(result, test.output)
    }

    fn is_first_word_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let result = is_first_word(&preprocessor_result);
        assert_eq!(result, test.output)
    }

    fn is_last_word_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let result = is_last_word(&preprocessor_result);
        assert_eq!(result, test.output)
    }

    fn contains_possessive_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let result = contains_possessive(&preprocessor_result);
        assert_eq!(result, test.output)
    }
}
