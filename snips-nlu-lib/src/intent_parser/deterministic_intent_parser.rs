use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use regex::{Regex, RegexBuilder};
use serde_json;

use builtin_entity_parsing::{BuiltinEntityParserFactory, CachingBuiltinEntityParser};
use errors::*;
use models::{FromPath, DeterministicParserModel};
use intent_parser::{internal_parsing_result, IntentParser, InternalParsingResult};
use language::FromLanguage;
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::{convert_to_char_range, substring_with_char_range, suffix_from_char_index};
use nlu_utils::token::{tokenize, tokenize_light};
use slot_utils::*;
use snips_nlu_ontology::Language;

pub struct DeterministicIntentParser {
    regexes_per_intent: HashMap<String, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
    builtin_entity_parser: Arc<CachingBuiltinEntityParser>,
    language: Language,
}

impl FromPath for DeterministicIntentParser {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(parser_model_path)?;
        let model: DeterministicParserModel = serde_json::from_reader(model_file)?;
        Self::new(model)
    }
}

impl DeterministicIntentParser {
    pub fn new(configuration: DeterministicParserModel) -> Result<Self> {
        let language = Language::from_str(&configuration.language_code)?;
        let builtin_entity_parser = BuiltinEntityParserFactory::get(language);
        Ok(DeterministicIntentParser {
            regexes_per_intent: compile_regexes_per_intent(configuration.patterns)?,
            group_names_to_slot_names: configuration.group_names_to_slot_names,
            slot_names_to_entities: configuration.slot_names_to_entities,
            builtin_entity_parser,
            language,
        })
    }
}

impl IntentParser for DeterministicIntentParser {
    fn parse(
        &self,
        input: &str,
        intents: Option<&HashSet<String>>,
    ) -> Result<Option<InternalParsingResult>> {
        let (ranges_mapping, formatted_input) =
            replace_builtin_entities(input, &*self.builtin_entity_parser);
        let language = NluUtilsLanguage::from_language(self.language);
        let cleaned_input = replace_tokenized_out_characters(input, language, ' ');
        let cleaned_formatted_input =
            replace_tokenized_out_characters(&*formatted_input, language, ' ');

        for (intent, regexes) in self.regexes_per_intent.iter() {
            if !intents
                .map(|intent_set| intent_set.contains(intent))
                .unwrap_or(true)
            {
                continue;
            }
            for regex in regexes {
                let matching_result_formatted = self.get_matching_result(
                    input,
                    &*cleaned_formatted_input,
                    regex,
                    intent,
                    Some(&ranges_mapping),
                );
                if matching_result_formatted.is_some() {
                    return Ok(matching_result_formatted);
                }
                let matching_result =
                    self.get_matching_result(input, &*cleaned_input, regex, intent, None);
                if matching_result.is_some() {
                    return Ok(matching_result);
                }
            }
        }
        Ok(None)
    }
}

impl DeterministicIntentParser {
    fn get_matching_result(
        &self,
        input: &str,
        formatted_input: &str,
        regex: &Regex,
        intent: &str,
        builtin_entities_ranges_mapping: Option<&HashMap<Range<usize>, Range<usize>>>,
    ) -> Option<InternalParsingResult> {
        if !regex.is_match(formatted_input) {
            return None;
        }

        for caps in regex.captures_iter(&formatted_input) {
            if caps.len() == 0 {
                continue;
            };

            let slots = caps.iter()
                .zip(regex.capture_names())
                .skip(1)
                .filter_map(|(opt_match, opt_group_name)| {
                    if let (Some(a_match), Some(group_name)) = (opt_match, opt_group_name) {
                        Some((a_match, group_name))
                    } else {
                        None
                    }
                })
                .map(|(a_match, group_name)| {
                    let slot_name = self.group_names_to_slot_names[group_name].to_string();
                    let entity = self.slot_names_to_entities[&slot_name].to_string();
                    let byte_range = a_match.start()..a_match.end();
                    let mut char_range = convert_to_char_range(&formatted_input, &byte_range);
                    if let Some(ranges_mapping) = builtin_entities_ranges_mapping {
                        char_range = ranges_mapping
                            .get(&char_range)
                            .map(|rng| rng.clone())
                            .unwrap_or_else(|| {
                                let shift = get_range_shift(&char_range, ranges_mapping);
                                let range_start = (char_range.start as i32 + shift) as usize;
                                let range_end = (char_range.end as i32 + shift) as usize;
                                range_start..range_end
                            });
                    }
                    let value = substring_with_char_range(input.to_string(), &char_range);
                    InternalSlot {
                        value,
                        char_range,
                        entity,
                        slot_name,
                    }
                })
                .collect();
            let deduplicated_slots = deduplicate_overlapping_slots(slots, self.language);
            let result = internal_parsing_result(intent.to_string(), 1.0, deduplicated_slots);
            return Some(result);
        }

        None
    }
}

fn compile_regexes_per_intent(
    patterns: HashMap<String, Vec<String>>,
) -> Result<HashMap<String, Vec<Regex>>> {
    patterns
        .into_iter()
        .map(|(intent, patterns)| {
            let regexes: Result<_> = patterns
                .into_iter()
                .map(|p| Ok(RegexBuilder::new(&p).case_insensitive(true).build()?))
                .collect();
            Ok((intent, regexes?))
        })
        .collect()
}

fn deduplicate_overlapping_slots(
    slots: Vec<InternalSlot>,
    language: Language,
) -> Vec<InternalSlot> {
    let mut deduped: Vec<InternalSlot> = Vec::with_capacity(slots.len());
    let language = NluUtilsLanguage::from_language(language);

    for slot in slots {
        let conflicting_slot_index = deduped
            .iter()
            .position(|existing_slot| ranges_overlap(&slot.char_range, &existing_slot.char_range));

        if let Some(index) = conflicting_slot_index {
            fn extract_counts(v: &InternalSlot, l: NluUtilsLanguage) -> (usize, usize) {
                (tokenize(&v.value, l).len(), v.value.chars().count())
            }
            let (existing_token_count, existing_char_count) =
                extract_counts(&deduped[index], language);
            let (token_count, char_count) = extract_counts(&slot, language);

            if token_count > existing_token_count
                || (token_count == existing_token_count && char_count > existing_char_count)
            {
                deduped[index] = slot;
            }
        } else {
            deduped.push(slot);
        }
    }
    deduped.sort_by_key(|slot| slot.char_range.start);
    deduped
}

fn replace_builtin_entities(
    text: &str,
    parser: &CachingBuiltinEntityParser,
) -> (HashMap<Range<usize>, Range<usize>>, String) {
    let builtin_entities = parser.extract_entities(text, None, true);
    if builtin_entities.is_empty() {
        return (HashMap::new(), text.to_string());
    }

    let mut range_mapping: HashMap<Range<usize>, Range<usize>> = HashMap::new();
    let mut processed_text = "".to_string();
    let mut offset = 0;
    let mut current_ix = 0;

    for entity in builtin_entities {
        let range_start = (entity.range.start as i16 + offset) as usize;
        let prefix_text =
            substring_with_char_range(text.to_string(), &(current_ix..entity.range.start));
        let entity_text = get_builtin_entity_name(entity.entity_kind.identifier());
        processed_text = format!("{}{}{}", processed_text, prefix_text, entity_text);
        offset += entity_text.chars().count() as i16 - entity.range.clone().count() as i16;
        let range_end = (entity.range.end as i16 + offset) as usize;
        let new_range = range_start..range_end;
        current_ix = entity.range.end;
        range_mapping.insert(new_range, entity.range);
    }

    processed_text = format!(
        "{}{}",
        processed_text,
        suffix_from_char_index(text.to_string(), current_ix)
    );
    (range_mapping, processed_text)
}

fn get_builtin_entity_name(entity_label: &str) -> String {
    // Here we don't need language specific tokenization, we just want to generate a feature name, that's why we use EN
    let normalized_entity_label = tokenize_light(entity_label, NluUtilsLanguage::EN)
        .join("")
        .to_uppercase();
    format!("%{}%", normalized_entity_label)
}

fn replace_tokenized_out_characters(
    string: &str,
    language: NluUtilsLanguage,
    replacement_char: char,
) -> String {
    let tokens = tokenize(string, language);
    let mut current_idx = 0;
    let mut cleaned_string = "".to_string();
    for token in tokens {
        let prefix_length = token.char_range.start - current_idx;
        let prefix: String = (0..prefix_length).map(|_| replacement_char).collect();
        cleaned_string = format!("{}{}{}", cleaned_string, prefix, token.value);
        current_idx = token.char_range.end;
    }
    let suffix_length = string.chars().count() - current_idx;
    let suffix: String = (0..suffix_length).map(|_| replacement_char).collect();
    cleaned_string = format!("{}{}", cleaned_string, suffix);
    cleaned_string
}

fn get_range_shift(
    matched_range: &Range<usize>,
    ranges_mapping: &HashMap<Range<usize>, Range<usize>>,
) -> i32 {
    let mut shift: i32 = 0;
    let mut previous_replaced_range_end: usize = 0;
    let match_start = matched_range.start;
    for (replaced_range, orig_range) in ranges_mapping.iter() {
        if replaced_range.end <= match_start {
            if replaced_range.end > previous_replaced_range_end {
                previous_replaced_range_end = replaced_range.end;
                shift = orig_range.end as i32 - replaced_range.end as i32;
            }
        }
    }
    shift
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::file_path;
    use models::DeterministicParserModel;
    use slot_utils::InternalSlot;
    use snips_nlu_ontology::{IntentClassifierResult, Language};
    use std::collections::HashMap;
    use std::iter::FromIterator;

    fn test_configuration() -> DeterministicParserModel {
        DeterministicParserModel {
            language_code: "en".to_string(),
            patterns: hashmap![
                "dummy_intent_1".to_string() => vec![
                    r"^This is a (?P<group_1>2 dummy a|dummy 2a|dummy_bb|dummy_a|dummy a|dummy_b|dummy b|dummy\d|dummy_3|dummy_1) query with another (?P<group_2>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.)$".to_string(),
                    r"^(?P<group_5>2 dummy a|dummy 2a|dummy_bb|dummy_a|dummy a|dummy_b|dummy b|dummy\d|dummy_3|dummy_1)$".to_string(),
                    r"^This is another (?P<group_3>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.) query.$".to_string(),
                    r"^This is another über (?P<group_3>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.) query.$".to_string(),
                    r"^This is another (?P<group_4>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.)?$".to_string(),
                ],
                "dummy_intent_2".to_string() => vec![
                    r"^This is a (?P<group_0>2 dummy a|dummy 2a|dummy_bb|dummy_a|dummy a|dummy_b|dummy b|dummy\d|dummy_3|dummy_1) query from another intent$".to_string()
                ],
                "dummy_intent_3".to_string() => vec![
                    r"^Send (?P<group_6>%SNIPSAMOUNTOFMONEY%) to john$".to_string(),
                    r"^Send (?P<group_6>%SNIPSAMOUNTOFMONEY%) to john at (?P<group_7>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.)$".to_string()
                ],
            ],
            group_names_to_slot_names: hashmap![
                "group_0".to_string() => "dummy_slot_name".to_string(),
                "group_1".to_string() => "dummy_slot_name".to_string(),
                "group_2".to_string() => "dummy_slot_name2".to_string(),
                "group_3".to_string() => "dummy_slot_name2".to_string(),
                "group_4".to_string() => "dummy_slot_name3".to_string(),
                "group_5".to_string() => "dummy_slot_name".to_string(),
                "group_6".to_string() => "dummy_slot_name4".to_string(),
                "group_7".to_string() => "dummy_slot_name2".to_string(),
            ],
            slot_names_to_entities: hashmap![
                "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                "dummy_slot_name3".to_string() => "dummy_entity_2".to_string(),
                "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
                "dummy_slot_name4".to_string() => "snips/amountOfMoney".to_string(),
            ],
        }
    }

    #[test]
    fn from_path_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("trained_engine")
            .join("deterministic_intent_parser");

        // When
        let intent_parser = DeterministicIntentParser::from_path(path).unwrap();
        let parsing_result = intent_parser.parse("make me two cups of coffee", None).unwrap();

        // Then
        let expected_intent = Some("MakeCoffee");
        let expected_slots= Some(vec![
            InternalSlot {
                value: "two".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string()
            }
        ]);
        assert_eq!(expected_intent, parsing_result.as_ref().map(|res| &*res.intent.intent_name));
        assert_eq!(expected_slots, parsing_result.map(|res| res.slots));
    }

    #[test]
    fn should_get_intent() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let intent = parser.parse(text, None).unwrap().map(|res| res.intent);

        // Then
        let expected_intent = Some(IntentClassifierResult {
            intent_name: "dummy_intent_1".to_string(),
            probability: 1.0,
        });

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn should_get_intent_with_builtin_entity() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "Send 10 dollars to John";

        // When
        let intent = parser.parse(text, None).unwrap().map(|res| res.intent);

        // Then
        let expected_intent = Some(IntentClassifierResult {
            intent_name: "dummy_intent_3".to_string(),
            probability: 1.0,
        });

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn should_get_slots() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let slots = parser.parse(text, None).unwrap().map(|res| res.slots);

        // Then
        let expected_slots = Some(vec![
            InternalSlot {
                value: "dummy_a".to_string(),
                char_range: 10..17,
                entity: "dummy_entity_1".to_string(),
                slot_name: "dummy_slot_name".to_string(),
            },
            InternalSlot {
                value: "dummy_c".to_string(),
                char_range: 37..44,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ]);
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_non_ascii_chars() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "This is another über dummy_cc query!";

        // When
        let slots = parser.parse(text, None).unwrap().map(|res| res.slots);

        // Then
        let expected_slots = Some(vec![
            InternalSlot {
                value: "dummy_cc".to_string(),
                char_range: 21..29,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ]);
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_builtin_entity() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "Send 10 dollars to John at dummy c";

        // When
        let slots = parser.parse(text, None).unwrap().map(|res| res.slots);

        // Then
        let expected_slots = Some(vec![
            InternalSlot {
                value: "10 dollars".to_string(),
                char_range: 5..15,
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "dummy_slot_name4".to_string(),
            },
            InternalSlot {
                value: "dummy c".to_string(),
                char_range: 27..34,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ]);
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_special_tokenized_out_characters() {
        // Given
        let parser = DeterministicIntentParser::new(test_configuration()).unwrap();
        let text = "this is another dummy’c";

        // When
        let slots = parser.parse(text, None).unwrap().map(|res| res.slots);

        // Then
        let expected_slots = Some(vec![
            InternalSlot {
                value: "dummy’c".to_string(),
                char_range: 16..23,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name3".to_string(),
            },
        ]);
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_deduplicate_overlapping_slots() {
        // Given
        let language = Language::EN;
        let slots = vec![
            InternalSlot {
                value: "non_overlapping1".to_string(),
                char_range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string(),
            },
            InternalSlot {
                value: "aaaaaaa".to_string(),
                char_range: 9..16,
                entity: "e1".to_string(),
                slot_name: "s2".to_string(),
            },
            InternalSlot {
                value: "bbbbbbbb".to_string(),
                char_range: 10..18,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "b cccc".to_string(),
                char_range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string(),
            },
            InternalSlot {
                value: "non_overlapping2".to_string(),
                char_range: 50..60,
                entity: "e".to_string(),
                slot_name: "s5".to_string(),
            },
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots, language);

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "non_overlapping1".to_string(),
                char_range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string(),
            },
            InternalSlot {
                value: "b cccc".to_string(),
                char_range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string(),
            },
            InternalSlot {
                value: "non_overlapping2".to_string(),
                char_range: 50..60,
                entity: "e".to_string(),
                slot_name: "s5".to_string(),
            },
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }

    #[test]
    fn should_replace_builtin_entities() {
        // Given
        let text = "Meeting this evening or tomorrow at 11am !";
        let parser = BuiltinEntityParserFactory::get(Language::EN);

        // When
        let (range_mapping, formatted_text) = replace_builtin_entities(text, &*parser);

        // Then
        let expected_mapping =
            HashMap::from_iter(vec![(8..23, 8..20), (27..42, 24..40)].into_iter());

        let expected_text = "Meeting %SNIPSDATETIME% or %SNIPSDATETIME% !";
        assert_eq!(expected_mapping, range_mapping);
        assert_eq!(expected_text, &formatted_text);
    }

    #[test]
    fn get_builtin_entity_name_works() {
        // Given
        let entity_label = "snips/datetime";

        // When
        let formatted_label = get_builtin_entity_name(entity_label);

        // Then
        assert_eq!("%SNIPSDATETIME%", &formatted_label)
    }

    #[test]
    fn should_replace_tokenized_out_characters() {
        // Given
        let string = ": hello, it's me !  ";
        let language = NluUtilsLanguage::EN;

        // When
        let cleaned_string = replace_tokenized_out_characters(string, language, '_');

        // Then
        assert_eq!("__hello__it_s_me_!__".to_string(), cleaned_string);
    }

    #[test]
    fn should_get_range_shift() {
        // Given
        let ranges_mapping = hashmap! {
            2..5 => 2..4,
            8..9 => 7..11
        };

        // When / Then
        assert_eq!(-1, get_range_shift(&(6..7), &ranges_mapping));
        assert_eq!(2, get_range_shift(&(12..13), &ranges_mapping));
    }
}
