use preprocessing::PreprocessorResult;
use models::gazetteer::Gazetteer;

pub fn has_gazetteer_hits<T: Gazetteer>(preprocessor_result: &PreprocessorResult,
                                        gazetteer: &T)
                                        -> Vec<f64> {
    match preprocessor_result.normalized_ngrams.iter().find(|ngram| gazetteer.contains(&ngram.0)) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

pub fn ngram_matcher(preprocessor_result: &PreprocessorResult, ngram_to_check: &str) -> Vec<f64> {
    match preprocessor_result.formatted_ngrams.iter().find(|ngram| ngram.0 == ngram_to_check) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

#[cfg(test)]
mod test {
    use std::ops::Range;
    use std::path;

    use super::has_gazetteer_hits;
    use super::ngram_matcher;
    use preprocessing::{NormalizedToken, PreprocessorResult};
    use preprocessing::convert_byte_index;
    use models::gazetteer::{HashSetGazetteer};
    use testutils::parse_json;
    use FileConfiguration;

    #[derive(Deserialize)]
    struct TestDescription {
        //description: String,
        input: Input,
        args: Vec<Arg>,
        output: f64,
    }

    #[derive(Deserialize)]
    struct Input {
        text: String,
        tokens: Vec<Token>,
    }

    #[derive(Deserialize)]
    struct Token {
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "endIndex")]
        end_index: usize,
        normalized: String,
        value: String,
        entity: Option<String>,
    }

    #[derive(Deserialize)]
    struct Arg {
        //#[serde(rename = "type")]
        //kind: String,
        //name: String,
        value: String,
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
        let file_configuration = FileConfiguration::default();

        let tests: Vec<(&str, Box<Fn(&FileConfiguration, &TestDescription, Vec<NormalizedToken>)>)> = vec![
            ("hasGazetteerHits", Box::new(has_gazetteer_hits_works)),
            ("ngramMatcher", Box::new(ngram_matcher_works)),
        ];

        let path = path::PathBuf::from("snips-sdk-tests/feature_extraction/SharedScalar");

        for test in tests {
            let test_name = test.0;
            let test_path = path.join(&test_name).with_extension("json");
            let parsed_tests: Vec<TestDescription> = parse_json(&test_path.to_str().unwrap());
            assert!(parsed_tests.len() != 0);

            for parsed_test in parsed_tests {
                let normalized_tokens: Vec<NormalizedToken> = parsed_test.input
                    .tokens
                    .iter()
                    .map(|test_token| test_token.to_normalized_token(&parsed_test.input.text))
                    .collect();

                test.1(&file_configuration, &parsed_test, normalized_tokens);
            }
        }
    }

    fn has_gazetteer_hits_works(file_configuration: &FileConfiguration, test: &TestDescription, normalized_tokens: Vec<NormalizedToken>) {
        let preprocessor_result = PreprocessorResult::new(normalized_tokens);
        let gazetteer = HashSetGazetteer::new(file_configuration, &*test.args[0].value).unwrap();
        let result = has_gazetteer_hits(&preprocessor_result, &gazetteer);
        assert_eq!(result, vec![test.output])
    }

    fn ngram_matcher_works(_: &FileConfiguration, test: &TestDescription, normalized_tokens: Vec<NormalizedToken>) {
        let preprocessor_result = PreprocessorResult::new(normalized_tokens);
        let result = ngram_matcher(&preprocessor_result, &test.args[0].value);
        assert_eq!(result, vec![test.output])
    }
}
