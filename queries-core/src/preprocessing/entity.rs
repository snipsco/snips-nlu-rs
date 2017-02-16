use std::ops::Range;
use std::iter::Iterator;

use regex::Regex;

use preprocessing::Token;
use preprocessing::convert_char_index;

struct EntityDetector {
    regex: &'static Regex,
    entity: &'static str,
}

impl EntityDetector {
    fn detect_entity_tokens(&self, input: &str) -> Vec<Token> {
        self.regex
            .captures_iter(input)
            .map(|capture| {
                let group = capture.pos(1).unwrap(); // TODO: yolo?
                Token {
                    value: capture.at(1).unwrap().to_string(),
                    range: Range {
                        start: group.0,
                        end: group.1,
                    },
                    char_range: Range {
                        start: convert_char_index(input, group.0),
                        end: convert_char_index(input, group.1),
                    },
                    entity: Some(self.entity.to_string()),
                }
            })
            .collect()
    }
}

lazy_static! {
    static ref ENTITY_DETECTORS : Vec<EntityDetector> = vec![date_detector(),
    digit_detector(),
    ordinal_detector(),
    price_detector(),
    temperature_detector(),
    time_detector()];
}


pub fn detect_entities(input: &str) -> Vec<Token> {
    let mut entity_tokens: Vec<Token> =
        ENTITY_DETECTORS.iter().flat_map(|detector| detector.detect_entity_tokens(input)).collect();

    let entity_ranges: Vec<Range<usize>> =
        entity_tokens.iter().map(|token| token.range.clone()).collect();

    entity_tokens.retain(|tested_token| {
        !entity_ranges.iter().any(|range| {
            range.start <= tested_token.range.start && range.end >= tested_token.range.end &&
            range.end - range.start > tested_token.range.end - tested_token.range.start
        })
    });


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

fn price_detector() -> EntityDetector {
    lazy_static! {
        static ref PRICE_REGEX: Regex = Regex::new(r"([$€£¥]\s?\d+(?:[\.,]\d+)?|\d+(?:[\.,]\d+)?\s?[$€£¥])").unwrap();
    }

    EntityDetector {
        regex: &(*PRICE_REGEX),
        entity: "%PRICE%",
    }
}


fn date_detector() -> EntityDetector {
    lazy_static! {
        static ref DATE_REGEX: Regex = Regex::new("\\b((?:(?:[0-2]?\\d|3[01])[/\\.-](?:[0-2]?\\d|3[01])[/\\.-](?:\\d+))|(?:\\d{4}[/\\.-](?:[0-2]?\\d|3[01])[/\\.-](?:[0-2]?\\d|3[01])))\\b").unwrap();
    }

    EntityDetector {
        regex: &(*DATE_REGEX),
        entity: "%DATE%",
    }
}

fn digit_detector() -> EntityDetector {
    lazy_static! {
        static ref DIGIT_REGEX: Regex = Regex::new(r"(?i)\b([\+-]?(?:\d+(?:[\.,]\d+)?|\d{1,3}(,\d{3})*(\.\d+)?|\.\d+)|(?:(?:(?:(?:(one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine))|(?:(ten)|(eleven)|(twelve)|(thirteen)|(fourteen)|(fifteen)|(sixteen)|(seventeen)|(eighteen)|(nineteen))|(?:(twenty)|(thirty)|(forty)|(fifty)|(sixty)|(seventy)|(eighty)|(ninety))\s(?:(one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine))|(?:(twenty)|(thirty)|(forty)|(fifty)|(sixty)|(seventy)|(eighty)|(ninety)))(?:\s((hundred)|(thousand)|(million)|(billion)|(trillion))(?:\sand\s|-|\s)))*(?:(?:(one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine))|(?:(ten)|(eleven)|(twelve)|(thirteen)|(fourteen)|(fifteen)|(sixteen)|(seventeen)|(eighteen)|(nineteen))|(?:(twenty)|(thirty)|(forty)|(fifty)|(sixty)|(seventy)|(eighty)|(ninety))\s(?:(one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine))|(?:(twenty)|(thirty)|(forty)|(fifty)|(sixty)|(seventy)|(eighty)|(ninety)))(?:\s(?:(hundred)|(thousand)|(million)|(billion)|(trillion)))?))\b").unwrap();
    }

    EntityDetector {
        regex: &(*DIGIT_REGEX),
        entity: "%DIGIT%",
    }
}


fn ordinal_detector() -> EntityDetector {
    lazy_static! {
        static ref ORDINAL_REGEX: Regex = Regex::new(r"(?i)\b((?:(?:[0-9]*1\s*st|[0-9]*2\s*nd|[0-9]*3\s*rd|[0-9]*[04-9]\s*th)|(?:(?:(?:(?:twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)(?:\s+|-))?(?:first|second|third|fourth|fifth|sixth|seventh|eighth|ninth))|(?:tenth|eleventh|twelfth|thirteenth|fourteenth|fifteenth|sixteenth|seventeenth|eighteenth|nineteenth|twentieth|thirtieth|fortieth|fiftieth|sixtieth|seventieth|eightieth|ninetieth))))\b").unwrap();
    }

    EntityDetector {
        regex: &(*ORDINAL_REGEX),
        entity: "%ORDINAL%",
    }
}

fn temperature_detector() -> EntityDetector {
    lazy_static! {
        static ref TEMPERATURE_REGEX: Regex = Regex::new(r"\b([+/-]?\d+(?:[\.,]\d+)?\s?°[CFK]?)\b").unwrap();
    }

    EntityDetector {
        regex: &(*TEMPERATURE_REGEX),
        entity: "%TEMPERATURE%",
    }
}

fn time_detector() -> EntityDetector {
    lazy_static! {
        static ref TIME_REGEX: Regex = Regex::new(r"((\b((?:(?:0?[1-9]|1[0-2])(?:[:\.][0-5]\d)?\s?[APap][mM])|(?:(?:[01]?\d|2[0-3])[:\.](?:[0-5]\d)))\b)|(((one)|(two)|(three)|(four)|(five)|(six)|(seven)|(eight)|(nine)|(ten)|(eleven)|(twelve)|(thirteen)|(fourteen)|(fifteen)|(sixteen)|(seventeen)|(eighteen)|(nineteen)|(twenty)|(thirty)|(fourty)|(fifty))((\so'?\s?clock)|(\s[ap]\.?[m]\.?))))").unwrap();
    }

    EntityDetector {
        regex: &(*TIME_REGEX),
        entity: "%TIME%",
    }
}


#[cfg(test)]
mod test {
    use testutils::parse_json;
    use super::price_detector;
    use super::date_detector;
    use super::digit_detector;
    use super::ordinal_detector;
    use super::temperature_detector;
    use super::time_detector;
    use super::EntityDetector;
    use super::detect_entities;
    use preprocessing::Token;

    #[derive(Deserialize, Debug)]
    struct TestDescription {
        description: String,
        input: String,
        output: Vec<TestOutput>,
    }

    #[derive(Deserialize, Debug)]
    struct TestOutput {
        entity: Option<String>,
        #[serde(rename = "startIndex")]
        start_index: usize,
        #[serde(rename = "endIndex")]
        end_index: usize,
        value: String,
    }

    #[test]
    fn price_detector_works() {
        do_entity_detection_test("price_detector.json", price_detector());
    }

    #[test]
    fn date_detector_works() {
        do_entity_detection_test("simple_date_detector.json", date_detector());
    }

    #[test]
    fn digit_detector_works() {
        do_entity_detection_test("fig_detector.json", digit_detector());
    }

    #[test]
    fn ordinal_detector_works() {
        do_entity_detection_test("ordinal_detector.json", ordinal_detector());
    }

    #[test]
    fn temperature_detector_works() {
        do_entity_detection_test("temperature_detector.json", temperature_detector());
    }

    #[test]
    fn time_detector_works() {
        do_entity_detection_test("time_detector.json", time_detector());
    }


    #[test]
    fn entity_tokenizer_works() {
        do_test("tokenization",
                "entity_tokenizer.json",
                &|input| detect_entities(input));
    }

    fn do_entity_detection_test(file_name: &str, detector: EntityDetector) {
        do_test("entity_detection",
                file_name,
                &|input| detector.detect_entity_tokens(input))
    }

    fn do_test(folder_name: &str, file_name: &str, tokenizer: &Fn(&str) -> Vec<Token>) {
        let path = format!("snips-sdk-tests/preprocessing/{}/{}",
                           folder_name,
                           file_name);
        let descs: Vec<TestDescription> = parse_json(&path);
        assert!(descs.len() != 0);
        for desc in descs {
            let result = tokenizer(&desc.input);
            assert_eq!(result.len(), desc.output.len());
            for (index, output) in desc.output.iter().enumerate() {
                assert_eq!(result[index].value, output.value);
                assert_eq!(result[index].entity, output.entity);
                assert_eq!(result[index].char_range.start, output.start_index);
                assert_eq!(result[index].char_range.end, output.end_index)
            }
        }
    }
}
