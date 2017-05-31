use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;
use regex::{Regex, RegexBuilder};

use errors::*;
use super::configuration::RuleBasedParserConfiguration;
use pipeline::{IntentClassifierResult, IntentParser, Slot};
use preprocessing::{tokenize, tokenize_light};
use utils::{substring_with_char_range, suffix_from_char_index, ranges_overlap};
use builtin_entities::RustlingParser;
use rustling_ontology::Lang;

pub struct RuleBasedIntentParser {
    regexes_per_intent: HashMap<String, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
    builtin_entity_parser: Arc<RustlingParser>
}

impl RuleBasedIntentParser {
    pub fn new(configuration: RuleBasedParserConfiguration) -> Result<Self> {
        let rustling_lang = Lang::from_str(&configuration.language)?;
        Ok(RuleBasedIntentParser {
            regexes_per_intent: compile_regexes_per_intent(configuration.regexes_per_intent)?,
            group_names_to_slot_names: configuration.group_names_to_slot_names,
            slot_names_to_entities: configuration.slot_names_to_entities,
            builtin_entity_parser: RustlingParser::get(rustling_lang)
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
        let (_, formatted_input) = replace_builtin_entities(input, &*self.builtin_entity_parser);
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

        let (ranges_mapping, formatted_input) = replace_builtin_entities(input, &*self.builtin_entity_parser);

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
                        let matched_range = a_match.start()..a_match.end();
                        let (value, range) = if let Some(rng) = ranges_mapping.get(&matched_range) {
                            (substring_with_char_range(input, rng), rng.clone())
                        } else {
                            (a_match.as_str().into(), matched_range)
                        };
                        let slot_name = self.group_names_to_slot_names[group_name].to_string();
                        let entity = self.slot_names_to_entities[&slot_name].to_string();

                        Slot { value, range, entity, slot_name }
                    })
                    .foreach(|slot| { result.push(slot); });
                break;
            }
        }
        Ok(deduplicate_overlapping_slots(result))
    }
}

fn deduplicate_overlapping_slots(slots: Vec<Slot>) -> Vec<Slot> {
    let mut deduped: Vec<Slot> = Vec::with_capacity(slots.len());

    for slot in slots {
        let conflicting_slot_index = deduped
            .iter()
            .position(|existing_slot| ranges_overlap(&slot.range, &existing_slot.range));

        if let Some(index) = conflicting_slot_index {
            fn extract_counts(v: &Slot) -> (usize, usize) {
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
    let builtin_entities = parser.extract_entities(text);
    if builtin_entities.len() == 0 {
        return (HashMap::new(), text.to_string())
    }

    let mut range_mapping: HashMap<Range<usize>, Range<usize>> = HashMap::new();
    let mut processed_text = "".to_string();
    let mut offset = 0;
    let mut current_ix = 0;

    for entity in builtin_entities {
        let range_start = (entity.char_range.start as i16 + offset) as usize;
        let prefix_text = substring_with_char_range(text, &(current_ix..entity.char_range.start));
        let entity_text = get_builtin_entity_name(entity.kind.identifier());
        processed_text = format!("{}{}{}", processed_text, prefix_text, entity_text);
        offset += entity_text.chars().count() as i16 - entity.value.chars().count() as i16;
        let range_end = (entity.char_range.end as i16 + offset) as usize;
        let new_range = range_start..range_end;
        current_ix = entity.char_range.end;
        range_mapping.insert(new_range, entity.char_range);
    }

    processed_text = format!("{}{}", processed_text, suffix_from_char_index(text, current_ix));
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
    use pipeline::rule_based::configuration::RuleBasedParserConfiguration;
    use pipeline::{IntentParser, IntentClassifierResult, Slot};
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
                    r"^This is another (?P<group_4>dummy_2_again|dummy_cc|dummy_c|dummy c|dummy_2|3p\.m\.)?$".to_string(),
                ],
                "dummy_intent_2".to_string() => vec![
                    r"^This is a (?P<group_0>2 dummy a|dummy 2a|dummy_bb|dummy_a|dummy a|dummy_b|dummy b|dummy\d|dummy_3|dummy_1) query from another intent$".to_string()
                ],
                "dummy_intent_3".to_string() => vec![
                    r"^meeting (?P<group_6>%SNIPSDATETIME%) with john$".to_string()
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
                "dummy_slot_name4".to_string() => "snips/datetime".to_string(),
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
        let text = "Meeting tomorrow night with John";

        // When
        let intent = parser.get_intent(text, None).unwrap().unwrap();

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: "dummy_intent_3".to_string(),
            probability: 1.0,
        };

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
            Slot {
                value: "dummy_a".to_string(),
                range: 10..17,
                entity: "dummy_entity_1".to_string(),
                slot_name: "dummy_slot_name".to_string()
            },
            Slot {
                value: "dummy_c".to_string(),
                range: 37..44,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string()
            }
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_get_slots_with_builtin_entity() {
        // Given
        let parser = RuleBasedIntentParser::new(test_configuration()).unwrap();
        let text = "Meeting tomorrow night with John";

        // When
        let slots = parser.get_slots(text, "dummy_intent_3").unwrap();

        // Then
        let expected_slots = vec![
            Slot {
                value: "tomorrow night".to_string(),
                range: 8..22,
                entity: "snips/datetime".to_string(),
                slot_name: "dummy_slot_name4".to_string()
            }
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn should_deduplicate_overlapping_slots() {
        // Given
        let slots = vec![
            Slot {
                value: "non_overlapping1".to_string(),
                range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string(),
            },
            Slot {
                value: "aaaaaaa".to_string(),
                range: 9..16,
                entity: "e1".to_string(),
                slot_name: "s2".to_string(),
            },
            Slot {
                value: "bbbbbbbb".to_string(),
                range: 10..18,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            Slot {
                value: "b cccc".to_string(),
                range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string(),
            },
            Slot {
                value: "non_overlapping2".to_string(),
                range: 50..60,
                entity: "e".to_string(),
                slot_name: "s5".to_string(),
            }
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots);

        // Then
        let expected_slots = vec![
            Slot {
                value: "non_overlapping1".to_string(),
                range: 3..7,
                entity: "e".to_string(),
                slot_name: "s1".to_string()
            },
            Slot {
                value: "b cccc".to_string(),
                range: 17..23,
                entity: "e1".to_string(),
                slot_name: "s4".to_string()
            },
            Slot {
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
        let text = "Meeting this evening or tomorrow morning !";
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
