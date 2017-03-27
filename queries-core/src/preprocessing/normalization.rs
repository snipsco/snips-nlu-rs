use std::ascii::AsciiExt;

use unicode_normalization::UnicodeNormalization;
use regex::Regex;
use yolo::Yolo;

pub fn normalize(input: &str) -> String {
    lazy_static! {
        static ref COMBINING_DIACRITRICAL_MARKS: Regex = Regex::new("[\u{0300}-\u{036F}]+").yolo();
    }

    COMBINING_DIACRITRICAL_MARKS.replace_all(&input.nfd().collect::<String>(), "")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use testutils::parse_json;
    use super::normalize;

    #[derive(Deserialize)]
    struct TestDescription {
        input: String,
        output: String,
    }

    #[test]
    fn normalization_works() {
        let descs: Vec<TestDescription> = parse_json("snips-sdk-tests/preprocessing/\
                                                      normalization/normalizer.json");
        assert!(descs.len() != 0);

        for desc in descs {
            let result = normalize(&desc.input);
            assert_eq!(result, desc.output)
        }
    }
}
