use preprocessing::PreprocessorResult;
use super::gazetteer::Gazetteer;

pub fn has_gazetteer_hits<T: Gazetteer>(preprocessed_result: &PreprocessorResult,
                                        gazetteer: T)
                                        -> Vec<f64> {
    match preprocessed_result.normalized_ngrams.iter().find(|ngram| gazetteer.contains(&ngram.0)) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

pub fn ngram_matcher(preprocessed_result: &PreprocessorResult, ngram_to_check: &str) -> Vec<f64> {
    match preprocessed_result.formatted_ngrams.iter().find(|ngram| ngram.0 == ngram_to_check) {
        Some(_) => vec![1.0],
        None => vec![0.0],
    }
}

#[cfg(test)]
mod test {
    use rustc_serialize::Decodable;
    use rustc_serialize::Decoder;
    use super::has_gazetteer_hits;
    use super::ngram_matcher;
    use preprocessing::preprocess;
    use preprocessing::PreprocessorResult;
    use models::gazetteer::Gazetteer;
    use models::gazetteer::HashSetGazetteer;
    use testutils::parse_json;

    #[derive(RustcDecodable)]
    struct TestDescription {
        description: String,
        input: Input,
        args: Vec<Arg>,
        output: f64,
    }

    #[derive(RustcDecodable)]
    struct Input {
        text: String,
        tokens: Vec<Token>,
    }

    #[derive(RustcDecodable)]
    struct Token {
        startIndex: i32,
        endIndex: i32,
        normalized: String,
        value: String,
    }

    struct Arg {
        kind: String,
        name: String,
        value: String,
    }

    impl Decodable for Arg {
        fn decode<D: Decoder>(d: &mut D) -> Result<Arg, D::Error> {
            d.read_struct("Arg", 2, |d| {
                let kind = try!(d.read_struct_field("type", 0, |d| d.read_str()));
                let name = try!(d.read_struct_field("name", 1, |d| d.read_str()));
                let value = try!(d.read_struct_field("value", 2, |d| d.read_str()));
                Ok(Arg {
                    kind: kind,
                    name: name,
                    value: value,
                })
            })
        }
    }

    #[test]
    fn has_gazetteer_hits_works() {
        let tests: Vec<TestDescription> = parse_json("../data/snips-sdk-tests/feature_extraction/SharedScalar/hasGazetteerHits.json");
        assert!(tests.len() != 0);
        for test in &tests {
            let gazetteer = HashSetGazetteer::new(&test.args[0].value).unwrap();
            let preprocessor_result = preprocess(&test.input.text);
            let result = has_gazetteer_hits(&preprocessor_result, gazetteer);
            assert_eq!(result, vec![test.output])
        }
    }

    #[test]
    fn ngram_matcher_works() {
        let tests: Vec<TestDescription> = parse_json("../data/snips-sdk-tests/feature_extraction/SharedScalar/ngramMatcher.json");
        assert!(tests.len() != 0);
        for test in &tests {
            let preprocessor_result = preprocess(&test.input.text);
            let result = ngram_matcher(&preprocessor_result, &test.args[0].value);
            assert_eq!(result, vec![test.output])
        }
    }
}
