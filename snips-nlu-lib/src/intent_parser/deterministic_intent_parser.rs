use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use regex::{Regex, RegexBuilder};
use serde_json;

use errors::*;
use failure::ResultExt;
use models::DeterministicParserModel;
use intent_parser::{internal_parsing_result, IntentParser, InternalParsingResult};
use language::FromLanguage;
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::{convert_to_char_range, substring_with_char_range, suffix_from_char_index};
use nlu_utils::token::{tokenize, tokenize_light};
use resources::SharedResources;
use slot_utils::*;
use snips_nlu_ontology::{BuiltinEntity, Language};
use utils::{deduplicate_overlapping_items, EntityName, IntentName, SlotName};

pub struct DeterministicIntentParser {
    language: Language,
    regexes_per_intent: HashMap<IntentName, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, SlotName>,
    slot_names_to_entities: HashMap<IntentName, HashMap<SlotName, EntityName>>,
    shared_resources: Arc<SharedResources>,
}

impl DeterministicIntentParser {
    pub fn from_path<P: AsRef<Path>>(path: P, shared_resources: Arc<SharedResources>) -> Result<Self> {
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(&parser_model_path)
            .with_context(|_|
                format!("Cannot open DeterministicIntentParser file '{:?}'",
                        &parser_model_path))?;
        let model: DeterministicParserModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize DeterministicIntentParser json data")?;
        Self::new(model, shared_resources)
    }
}

impl DeterministicIntentParser {
    pub fn new(
        configuration: DeterministicParserModel,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let language = Language::from_str(&configuration.language_code)?;
        Ok(DeterministicIntentParser {
            language,
            regexes_per_intent: compile_regexes_per_intent(configuration.patterns)?,
            group_names_to_slot_names: configuration.group_names_to_slot_names,
            slot_names_to_entities: configuration.slot_names_to_entities,
            shared_resources,
        })
    }
}

impl IntentParser for DeterministicIntentParser {
    fn parse(
        &self,
        input: &str,
        intents: Option<&HashSet<IntentName>>,
    ) -> Result<Option<InternalParsingResult>> {
        let builtin_entities = self.shared_resources
            .builtin_entity_parser
            .extract_entities(input, None, true)?;
        let (ranges_mapping, formatted_input) = replace_builtin_entities(input, builtin_entities);
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
                    let entity = self.slot_names_to_entities[intent][&slot_name].to_string();
                    let byte_range = a_match.start()..a_match.end();
                    let mut char_range = convert_to_char_range(&formatted_input, &byte_range);
                    if let Some(ranges_mapping) = builtin_entities_ranges_mapping {
                        char_range = ranges_mapping
                            .get(&char_range)
                            .cloned()
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
    patterns: HashMap<IntentName, Vec<String>>,
) -> Result<HashMap<IntentName, Vec<Regex>>> {
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
    let language = NluUtilsLanguage::from_language(language);
    let slots_overlap = |lhs_slot: &InternalSlot, rhs_slot: &InternalSlot| {
        ranges_overlap(&lhs_slot.char_range, &rhs_slot.char_range)
    };
    let slot_sort_key = |slot: &InternalSlot| {
        let tokens_count = tokenize(&slot.value, language).len();
        let chars_count = slot.value.chars().count();
        -((tokens_count + chars_count) as i32)
    };
    let mut deduped = deduplicate_overlapping_items(
        slots, slots_overlap, slot_sort_key);
    deduped.sort_by_key(|slot| slot.char_range.start);
    deduped
}

fn deduplicate_overlapping_entities(entities: Vec<BuiltinEntity>) -> Vec<BuiltinEntity> {
    let entities_overlap = |lhs_entity: &BuiltinEntity, rhs_entity: &BuiltinEntity| {
        ranges_overlap(&lhs_entity.range, &rhs_entity.range)
    };
    let entity_sort_key = |entity: &BuiltinEntity| -(entity.range.clone().count() as i32);
    let mut deduped = deduplicate_overlapping_items(entities, entities_overlap, entity_sort_key);
    deduped.sort_by_key(|entity| entity.range.start);
    deduped
}

fn replace_builtin_entities(
    text: &str,
    builtin_entities: Vec<BuiltinEntity>,
) -> (HashMap<Range<usize>, Range<usize>>, String) {
    if builtin_entities.is_empty() {
        return (HashMap::new(), text.to_string());
    }

    let mut dedup_entities = deduplicate_overlapping_entities(builtin_entities);
    dedup_entities.sort_by_key(|entity| entity.range.start);

    let mut range_mapping: HashMap<Range<usize>, Range<usize>> = HashMap::new();
    let mut processed_text = "".to_string();
    let mut offset = 0;
    let mut current_ix = 0;

    for entity in dedup_entities {
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
        if replaced_range.end <= match_start &&
            replaced_range.end > previous_replaced_range_end {
            previous_replaced_range_end = replaced_range.end;
            shift = orig_range.end as i32 - replaced_range.end as i32;
        }
    }
    shift
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use super::*;

    use models::DeterministicParserModel;
    use slot_utils::InternalSlot;
    use snips_nlu_ontology::*;
    use testutils::*;

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
                "dummy_intent_1".to_string() => hashmap![
                    "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                    "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
                    "dummy_slot_name3".to_string() => "dummy_entity_2".to_string(),
                ],
                "dummy_intent_2".to_string() => hashmap![
                    "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                ],
                "dummy_intent_3".to_string() => hashmap![
                    "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
                    "dummy_slot_name4".to_string() => "snips/amountOfMoney".to_string(),
                ],
            ],
        }
    }

    #[test]
    fn from_path_works() {
        // Given
        let trained_engine_path = file_path("tests")
            .join("models")
            .join("nlu_engine");

        let parser_path = trained_engine_path.join("deterministic_intent_parser");

        let shared_resources = load_shared_resources_from_engine_dir(trained_engine_path).unwrap();
        let intent_parser = DeterministicIntentParser::from_path(parser_path, shared_resources).unwrap();

        // When
        let parsing_result = intent_parser.parse("make me two cups of coffee", None).unwrap();

        // Then
        let expected_intent = Some("MakeCoffee");
        let expected_slots = Some(vec![
            InternalSlot {
                value: "two".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            }
        ]);
        assert_eq!(expected_intent, parsing_result.as_ref().map(|res| &*res.intent.intent_name));
        assert_eq!(expected_slots, parsing_result.map(|res| res.slots));
    }

    #[test]
    fn should_get_intent() {
        // Given
        let shared_resources = SharedResourcesBuilder::default().build();
        let parser = DeterministicIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();
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
        let text = "Send 10 dollars to John";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(
            vec![(
                text.to_string(),
                vec![
                    BuiltinEntity {
                        value: "10 dollars".to_string(),
                        range: 5..15,
                        entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                            value: 10.,
                            precision: Precision::Exact,
                            unit: Some("dollars".to_string()),
                        }),
                        entity_kind: BuiltinEntityKind::AmountOfMoney,
                    }
                ]
            )]
        );
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .build();
        let parser = DeterministicIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

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
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = DeterministicIntentParser::new(test_configuration(), shared_resources).unwrap();
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
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = DeterministicIntentParser::new(test_configuration(), shared_resources).unwrap();
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
        let text = "Send 10 dollars to John at dummy c";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(
            vec![(
                text.to_string(),
                vec![
                    BuiltinEntity {
                        value: "10 dollars".to_string(),
                        range: 5..15,
                        entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                            value: 10.,
                            precision: Precision::Exact,
                            unit: Some("dollars".to_string()),
                        }),
                        entity_kind: BuiltinEntityKind::AmountOfMoney,
                    }
                ]
            )]
        );
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .build();
        let parser = DeterministicIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

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
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = DeterministicIntentParser::new(test_configuration(), shared_resources).unwrap();
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
                value: "kid".to_string(),
                char_range: 0..3,
                entity: "e1".to_string(),
                slot_name: "s1".to_string(),
            },
            InternalSlot {
                value: "loco".to_string(),
                char_range: 4..8,
                entity: "e1".to_string(),
                slot_name: "s2".to_string(),
            },
            InternalSlot {
                value: "kid loco".to_string(),
                char_range: 0..8,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "song".to_string(),
                char_range: 9..13,
                entity: "e2".to_string(),
                slot_name: "s4".to_string(),
            },
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots, language);

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "kid loco".to_string(),
                char_range: 0..8,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "song".to_string(),
                char_range: 9..13,
                entity: "e2".to_string(),
                slot_name: "s4".to_string(),
            },
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }

    #[test]
    fn should_replace_builtin_entities() {
        // Given
        let text = "the third album of Blink 182 is great";
        let builtin_entities = vec![
            BuiltinEntity {
                value: "the third".to_string(),
                range: 0..9,
                entity: SlotValue::Ordinal(OrdinalValue { value: 3 }),
                entity_kind: BuiltinEntityKind::Ordinal,
            },
            BuiltinEntity {
                value: "182".to_string(),
                range: 25..28,
                entity: SlotValue::Number(NumberValue { value: 182.0 }),
                entity_kind: BuiltinEntityKind::Number,
            },
            BuiltinEntity {
                value: "Blink 182".to_string(),
                range: 19..28,
                entity: SlotValue::MusicArtist(StringValue { value: "Blink 182".to_string() }),
                entity_kind: BuiltinEntityKind::MusicArtist,
            },
        ];

        // When
        let (range_mapping, formatted_text) = replace_builtin_entities(text, builtin_entities);

        // Then
        let expected_mapping = HashMap::from_iter(vec![(0..14, 0..9), (24..42, 19..28)]);

        let expected_text = "%SNIPSORDINAL% album of %SNIPSMUSICARTIST% is great";
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
