use std::collections::{HashMap, HashSet};
use std::ops::Range;

use itertools::Itertools;
use regex::{Regex, RegexBuilder};

use errors::*;
use super::configuration::RuleBasedParserConfiguration;
use pipeline::{IntentClassifierResult, IntentParser, Slot};
use preprocessing::tokenize;

pub struct RuleBasedIntentParser {
    regexes_per_intent: HashMap<String, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
}

impl RuleBasedIntentParser {
    pub fn new(configuration: RuleBasedParserConfiguration) -> Result<Self> {
        Ok(RuleBasedIntentParser {
            regexes_per_intent: compile_regexes_per_intent(configuration.regexes_per_intent)?,
            group_names_to_slot_names: configuration.group_names_to_slot_names,
            slot_names_to_entities: configuration.slot_names_to_entities,
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
        Ok(self.regexes_per_intent.iter()
            .filter(|&(intent, _)|
                if let Some(intent_set) = intents {
                    intent_set.contains(intent)
                } else {
                    true
                }
            )
            .find(|&(_, regexes)| regexes.iter().find(|r| r.is_match(input)).is_some())
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

        let mut result = vec![];
        for regex in regexes {
            for caps in regex.captures_iter(input) {
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
                        let range = a_match.start()..a_match.end();
                        let value = a_match.as_str().into();
                        let slot_name = self.group_names_to_slot_names[group_name].to_string();
                        let entity = self.slot_names_to_entities[&slot_name].to_string();

                        Slot { value, range, entity, slot_name }
                    })
                    .foreach(|slot| { result.push(slot); });
            }
        }
        Ok(deduplicate_overlapping_slots(result))
    }
}

fn are_overlapping(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r1.end > r2.start && r1.start < r2.end
}

fn deduplicate_overlapping_slots(slots: Vec<Slot>) -> Vec<Slot> {
    let mut deduped: Vec<Slot> = Vec::with_capacity(slots.len());

    for slot in slots {
        let conflicting_slot_index = deduped
            .iter()
            .position(|existing_slot| are_overlapping(&slot.range, &existing_slot.range));

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

#[cfg(test)]
mod tests {
    use pipeline::rule_based::configuration::RuleBasedParserConfiguration;
    use super::RuleBasedIntentParser;
    use super::deduplicate_overlapping_slots;
    use pipeline::{IntentParser, IntentClassifierResult, Slot};

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
                ]
            ],
            group_names_to_slot_names: hashmap![
                "group_0".to_string() => "dummy_slot_name".to_string(),
                "group_1".to_string() => "dummy_slot_name".to_string(),
                "group_2".to_string() => "dummy_slot_name2".to_string(),
                "group_3".to_string() => "dummy_slot_name2".to_string(),
                "group_4".to_string() => "dummy_slot_name3".to_string(),
                "group_5".to_string() => "dummy_slot_name".to_string(),
            ],
            slot_names_to_entities: hashmap![
                "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                "dummy_slot_name3".to_string() => "dummy_entity_2".to_string(),
                "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
            ],
        }
    }

    #[test]
    fn test_should_get_intent() {
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
    fn test_should_get_entities() {
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

    // TODO test will fail if you inverse lines s3 and s4 (same as in python)
    #[test]
    fn test_should_deduplicate_overlapping_slots() {
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
}
