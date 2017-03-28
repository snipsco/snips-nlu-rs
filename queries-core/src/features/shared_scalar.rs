use preprocessing::PreprocessorResult;
use models::gazetteer::Gazetteer;

pub fn has_gazetteer_hits(preprocessor_result: &PreprocessorResult,
                          gazetteer: Box<Gazetteer>)
                          -> Vec<f32> {
    match preprocessor_result.normalized_ngrams.iter().find(|ngram| gazetteer.contains(&ngram.0)) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

pub fn ngram_matcher(preprocessor_result: &PreprocessorResult, ngram_to_check: &str) -> Vec<f32> {
    match preprocessor_result.formatted_ngrams.iter().find(|ngram| ngram.0 == ngram_to_check) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

pub fn get_message_length(preprocessor_result: &PreprocessorResult,
                          normalization: f32)
                          -> Vec<f32> {
    vec![preprocessor_result.raw.chars().count() as f32 / normalization]
}

#[cfg(test)]
mod test {
    use std::ops::Range;
    use std::path;

    use serde_json;

    use super::has_gazetteer_hits;
    use super::ngram_matcher;
    use super::get_message_length;

    use config::FileBasedAssistantConfig;
    use models::gazetteer::HashSetGazetteer;
    use preprocessing::{NormalizedToken, PreprocessorResult};
    use preprocessing::convert_byte_index;
    use testutils::parse_json;

    #[derive(Debug, Deserialize)]
    struct TestDescription {
        //description: String,
        input: Input,
        args: Vec<Arg>,
        output: f32,
    }

    #[derive(Debug, Deserialize)]
    struct Input {
        text: String,
        tokens: Vec<Token>,
    }

    #[derive(Debug, Deserialize)]
    struct Token {
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "endIndex")]
        end_index: usize,
        normalized: String,
        value: String,
        entity: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Arg {
        //#[serde(rename = "type")]
        //kind: String,
        //name: String,
        value: Data,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Data {
        StringValue(String),
        StringArray(Vec<String>),
        Float(f32),
    }

    impl Token {
        fn to_normalized_token(&self, base_string: &str) -> NormalizedToken {
            NormalizedToken {
                value: self.value.clone(),
                normalized_value: self.normalized.clone(),
                range: Range {
                    start: convert_byte_index(base_string, self.start_index),
                    end: convert_byte_index(base_string, self.end_index),
                },
                char_range: Range {
                    start: self.start_index,
                    end: self.end_index,
                },
                entity: self.entity.clone(),
            }
        }
    }

    #[test]
    fn feature_function_works() {
        let tests: Vec<(&str, Box<Fn(&TestDescription, &PreprocessorResult)>)> = vec![
            ("hasGazetteerHits", Box::new(has_gazetteer_hits_works)),
            ("ngramMatcher", Box::new(ngram_matcher_works)),
            //("getMessageLength", Box::new(get_message_length_works)),
        ];

        let path = path::PathBuf::from("snips-sdk-tests/feature_extraction/SharedScalar");

        for test in tests {
            let test_name = test.0;
            let test_path = path.join(&test_name).with_extension("json");
            let parsed_tests: Vec<TestDescription> = parse_json(&test_path.to_str().unwrap());
            assert!(parsed_tests.len() != 0);

            for parsed_test in parsed_tests {
                let input_text = parsed_test.input.text.to_string();
                let normalized_tokens: Vec<NormalizedToken> = parsed_test.input.tokens
                    .iter()
                    .map(|test_token| test_token.to_normalized_token(&parsed_test.input.text))
                    .collect();

                let preprocessor_result = PreprocessorResult::new(input_text, normalized_tokens);
                test.1(&parsed_test, &preprocessor_result);
            }
        }
    }

    fn has_gazetteer_hits_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let gazetteer_values = if let Data::StringArray(ref value) = test.args[0].value {
           value
        } else {
            panic!("test doesn't contain a gazetteer: {:?}", test)
        };

        //let gazetteer = HashSetGazetteer::new(gazetteer_values).unwrap();
        //let result = has_gazetteer_hits(&preprocessor_result, &gazetteer);
        //assert_eq!(result, vec![test.output])
    }

    fn ngram_matcher_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let ngram = if let Data::StringValue(ref value) = test.args[0].value { value } else { panic!() };
        let result = ngram_matcher(&preprocessor_result, &ngram);
        assert_eq!(result, vec![test.output])
    }

    fn get_message_length_works(test: &TestDescription, preprocessor_result: &PreprocessorResult) {
        let normalization = if let Data::Float(value) = test.args[0].value { value } else { panic!() };
        let result = get_message_length(&preprocessor_result, normalization);
        assert_eq!(result, vec![test.output])
    }
}
