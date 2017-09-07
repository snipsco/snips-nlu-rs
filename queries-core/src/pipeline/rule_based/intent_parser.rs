use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;
use regex::{Regex, RegexBuilder};

use errors::*;
use super::configuration::RuleBasedParserConfiguration;
use snips_queries_ontology::{IntentClassifierResult, Slot};
use pipeline::{IntentParser, InternalSlot};
use pipeline::slot_utils::{resolve_builtin_slots, convert_to_custom_slot};
use nlu_utils::token::{tokenize, tokenize_light};
use nlu_utils::string::{convert_to_char_range, substring_with_char_range, suffix_from_char_index};
use nlu_utils::range::ranges_overlap;
use builtin_entities::{BuiltinEntityKind, RustlingParser};
use rustling_ontology::Lang;

pub struct RuleBasedIntentParser {
    regexes_per_intent: HashMap<String, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
    builtin_entity_parser: Option<Arc<RustlingParser>>
}

impl RuleBasedIntentParser {
    pub fn new(configuration: RuleBasedParserConfiguration) -> Result<Self> {
        let builtin_entity_parser = Lang::from_str(&configuration.language).ok()
            .map(|rustling_lang| RustlingParser::get(rustling_lang));
        Ok(RuleBasedIntentParser {
            regexes_per_intent: compile_regexes_per_intent(configuration.regexes_per_intent)?,
            group_names_to_slot_names: configuration.group_names_to_slot_names,
            slot_names_to_entities: configuration.slot_names_to_entities,
            builtin_entity_parser
        })
    }
}

fn compile_regexes_per_intent(patterns: HashMap<String, Vec<String>>)
                              -> Result<HashMap<String, Vec<Regex>>> {
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

impl IntentParser for RuleBasedIntentParser {
    fn get_intent(&self, input: &str, intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>> {
        let formatted_input = if let Some(builtin_entity_parser) = self.builtin_entity_parser.as_ref() {
            replace_builtin_entities(input, &*builtin_entity_parser).1
        } else {
            input.to_string()
        };
        Ok(self.regexes_per_intent.iter()
            .filter(|&(intent, _)|
                if let Some(intent_set) = intents {
                    intent_set.contains(intent)
                } else {
                    true
                }
            )
            .find(|&(_, regexes)| regexes.iter().find(|r| r.is_match(&formatted_input)).is_some())
            .map(|(intent_name, _)| {
                IntentClassifierResult {
                    intent_name: intent_name.to_string(),
                    probability: 1.0,
                }
            })
        )
    }

    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>> {
        let regexes = self.regexes_per_intent
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;

        let (ranges_mapping, formatted_input) = if let Some(builtin_entity_parser) = self.builtin_entity_parser.as_ref() {
            replace_builtin_entities(input, &*builtin_entity_parser)
        } else {
            (HashMap::<Range<usize>, Range<usize>>::new(), input.to_string())
        };

        let mut result = vec![];
        for regex in regexes {
            for caps in regex.captures_iter(&formatted_input) {
                if caps.len() == 0 { continue };
                caps.iter()
                    .zip(regex.capture_names())
                    .skip(1)
                    .filter_map(|(opt_match, opt_group_name)|
                        if let (Some(a_match), Some(group_name)) = (opt_match, opt_group_name) {
                            Some((a_match, group_name))
                        } else {
                            None
                        })
                    .map(|(a_match, group_name)| {
                        let byte_range = a_match.start()..a_match.end();
                        let matched_range = convert_to_char_range(&formatted_input, &byte_range);
                        let (value, range) = if let Some(rng) = ranges_mapping.get(&matched_range) {
                            (substring_with_char_range(input.to_string(), rng), rng.clone())
                        } else {
                            (a_match.as_str().into(), matched_range)
                        };
                        let slot_name = self.group_names_to_slot_names[group_name].to_string();
                        let entity = self.slot_names_to_entities[&slot_name].to_string();

                        InternalSlot { value, range, entity, slot_name }
                    })
                    .foreach(|slot| { result.push(slot); });
                break;
            }
        }
        let deduplicated_slots = deduplicate_overlapping_slots(result);
        if let Some(builtin_entity_parser) = self.builtin_entity_parser.as_ref() {
            let filter_entity_kinds = self.slot_names_to_entities
                .values()
                .flat_map(|entity_name| BuiltinEntityKind::from_identifier(entity_name).ok())
                .unique()
                .collect::<Vec<_>>();
            Ok(resolve_builtin_slots(input, deduplicated_slots, &*builtin_entity_parser, Some(&*filter_entity_kinds)))
        } else {
            Ok(deduplicated_slots.into_iter()
                .filter_map(|s| {
                    if BuiltinEntityKind::from_identifier(&s.entity).is_err() {
                        Some(convert_to_custom_slot(s))
                    } else {
                        None
                    }
                })
                .collect())
        }
    }
}

fn deduplicate_overlapping_slots(slots: Vec<InternalSlot>) -> Vec<InternalSlot> {
    let mut deduped: Vec<InternalSlot> = Vec::with_capacity(slots.len());

    for slot in slots {
        let conflicting_slot_index = deduped
            .iter()
            .position(|existing_slot| ranges_overlap(&slot.range, &existing_slot.range));

        if let Some(index) = conflicting_slot_index {
            fn extract_counts(v: &InternalSlot) -> (usize, usize) {
                (tokenize(&v.value).len(), v.value.chars().count())
            }
            let (existing_token_count, existing_char_count) = extract_counts(&deduped[index]);
            let (token_count, char_count) = extract_counts(&slot);

            if token_count > existing_token_count ||
                (token_count == existing_token_count && char_count > existing_char_count) {
                deduped[index] = slot;
            }
        } else {
            deduped.push(slot);
        }
    }
    deduped.sort_by_key(|slot| slot.range.start);
    deduped
}

fn replace_builtin_entities(text: &str,
                            parser: &RustlingParser) -> (HashMap<Range<usize>, Range<usize>>, String) {
    let builtin_entities = parser.extract_entities(text, None);
    if builtin_entities.len() == 0 {
        return (HashMap::new(), text.to_string());
    }

    let mut range_mapping: HashMap<Range<usize>, Range<usize>> = HashMap::new();
    let mut processed_text = "".to_string();
    let mut offset = 0;
    let mut current_ix = 0;

    for entity in builtin_entities {
        let range_start = (entity.range.start as i16 + offset) as usize;
        let prefix_text = substring_with_char_range(text.to_string(), &(current_ix..entity.range.start));
        let entity_text = get_builtin_entity_name(entity.entity_kind.identifier());
        processed_text = format!("{}{}{}", processed_text, prefix_text, entity_text);
        offset += entity_text.chars().count() as i16 - entity.range.clone().count() as i16;
        let range_end = (entity.range.end as i16 + offset) as usize;
        let new_range = range_start..range_end;
        current_ix = entity.range.end;
        range_mapping.insert(new_range, entity.range);
    }

    processed_text = format!("{}{}", processed_text, suffix_from_char_index(text.to_string(), current_ix));
    (range_mapping, processed_text)
}

fn get_builtin_entity_name(entity_label: &str) -> String {
    let normalized_entity_label = tokenize_light(entity_label).join("").to_uppercase();
    format!("%{}%", normalized_entity_label)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use snips_queries_ontology::{AmountOfMoneyValue, IntentClassifierResult, Precision,Slot, SlotValue};
    use pipeline::rule_based::configuration::RuleBasedParserConfiguration;
    use pipeline::{IntentParser, InternalSlot};
    use builtin_entities::RustlingParser;
    use rustling_ontology::Lang;
    use super::{RuleBasedIntentParser, deduplicate_overlapping_slots, get_builtin_entity_name,
                replace_builtin_entities};

    fn test_configuration() -> RuleBasedParserConfiguration {
        RuleBasedParserConfiguration {
            language: "en".to_string(),
            regexes_per_intent: hashmap![
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
                    r"^Send (?P<group_6>%SNIPSAMOUNTOFMONEY%) to john$".to_string()
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
    fn should_get_intent() {
        // Given
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let intent = parser.get_intent(text, None).unwrap().unwrap();

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: "dummy_intent_1".to_string(),
            probability: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn should_get_intent_with_builtin_entity() {
        // Given
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "Send 10 dollars to John";

        // When
        let intent = parser.get_intent(text, None).unwrap();

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
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let slots = parser.get_slots(text, "dummy_intent_1").unwrap();

        // Then
        let expected_slots = vec![
            Slot::new_custom(
                "dummy_a".to_string(),
                10..17,
                "dummy_entity_1".to_string(),
                "dummy_slot_name".to_string()
            ),
            Slot::new_custom(
                "dummy_c".to_string(),
                37..44,
                "dummy_entity_2".to_string(),
                "dummy_slot_name2".to_string()
            ),
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_non_ascii_chars() {
        // Given
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "This is another über dummy_cc query!";


        // When
        let slots = parser.get_slots(text, "dummy_intent_1").unwrap();

        // Then
        let expected_slots = vec![
            Slot::new_custom(
                "dummy_cc".to_string(),
                21..29,
                "dummy_entity_2".to_string(),
                "dummy_slot_name2".to_string()
            ),
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_builtin_entity() {
        // Given
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "Send 10 dollars to John";

        // When
        let slots = parser.get_slots(text, "dummy_intent_3").unwrap();

        // Then
        let expected_slots = vec![
            Slot {
                raw_value: "10 dollars".to_string(),
                value: SlotValue::AmountOfMoney(
                    AmountOfMoneyValue {
                        value: 10.0,
                        precision: Precision::Exact,
                        unit: Some("$".to_string())
                    }
                ),
                range: Some(5..15),
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "dummy_slot_name4".to_string(),
            }
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_deduplicate_overlapping_slots() {
        // Given
        let slots = vec![
            InternalSlot {
                value: "non_overlapping1".to_string(),
                range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string(),
            },
            InternalSlot {
                value: "aaaaaaa".to_string(),
                range: 9..16,
                entity: "e1".to_string(),
                slot_name: "s2".to_string(),
            },
            InternalSlot {
                value: "bbbbbbbb".to_string(),
                range: 10..18,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "b cccc".to_string(),
                range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string(),
            },
            InternalSlot {
                value: "non_overlapping2".to_string(),
                range: 50..60,
                entity: "e".to_string(),
                slot_name: "s5".to_string(),
            },
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots);

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "non_overlapping1".to_string(),
                range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string()
            },
            InternalSlot {
                value: "b cccc".to_string(),
                range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string()
            },
            InternalSlot {
                value: "non_overlapping2".to_string(),
                range: 50..60,
                entity: "e".to_string(),
                slot_name: "s5".to_string()
            },
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }

    #[test]
    fn should_replace_builtin_entities() {
        // Given
        let text = "Meeting this evening or tomorrow at 11am !";
        let parser = RustlingParser::get(Lang::EN);

        // When
        let (range_mapping, formatted_text) = replace_builtin_entities(text, &*parser);

        // Then
        let expected_mapping = HashMap::from_iter(
            vec![
                (8..23, 8..20),
                (27..42, 24..40)
            ].into_iter()
        );

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
}
