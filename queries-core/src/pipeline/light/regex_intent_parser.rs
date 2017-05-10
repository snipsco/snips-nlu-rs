use std::collections::HashMap;
use std::ops::Range;

use itertools::Itertools;
use regex::{Regex, RegexBuilder};

use errors::*;
use super::configuration::RegexIntentParserConfiguration;
use pipeline::{IntentClassifierResult, IntentParser, IntentParserResult, Slots, SlotValue};
use preprocessing::light::tokenize;

pub struct RegexIntentParser {
    regexes_per_intent: HashMap<String, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
}

impl RegexIntentParser {
    pub fn new(configuration: RegexIntentParserConfiguration) -> Result<Self> {
        Ok(RegexIntentParser {
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

impl IntentParser for RegexIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        let classif_results = self.get_intent(input, probability_threshold)?;

        if let Some(best_classif) = classif_results.first() {
            let intent_name = best_classif.intent_name.to_string();
            let likelihood = best_classif.probability;
            let slots = self.get_entities(input, &intent_name)?;

            Ok(Some(IntentParserResult {
                        input: input.to_string(),
                        likelihood,
                        intent_name,
                        slots,
                    }))
        } else {
            Ok(None)
        }
    }

    fn get_intent(&self, input: &str, _: f32) -> Result<Vec<IntentClassifierResult>> {
        let result = self.regexes_per_intent
            .iter()
            .find(|&(_, regexes)| regexes.iter().find(|r| r.is_match(input)).is_some())
            .map(|(intent_name, _)| {
                     IntentClassifierResult {
                         intent_name: intent_name.to_string(),
                         probability: 1.0,
                     }
                 });

        if let Some(best_classif) = result {
            Ok(vec![best_classif])
        } else {
            Ok(vec![])
        }
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        let regexes = self.regexes_per_intent
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;

        let mut result = vec![];
        for regex in regexes {
            for caps in regex.captures_iter(input) {
                caps.iter()
                    .zip(regex.capture_names())
                    .skip(1)
                    .filter_map(|(opt_match, opt_group_name)| if let (Some(a_match),
                                                                      Some(group_name)) =
                        (opt_match, opt_group_name) {
                                    Some((a_match, group_name))
                                } else {
                                    None
                                })
                    .map(|(a_match, group_name)| {
                        let range = a_match.start()..a_match.end();
                        let value = a_match.as_str().into();
                        let slot_name = self.group_names_to_slot_names[group_name].to_string();
                        let entity = self.slot_names_to_entities[&slot_name].to_string();

                        (slot_name,
                         SlotValue {
                             value,
                             range,
                             entity,
                         })
                    })
                    .foreach(|(slot_name, slot_value)| { result.push((slot_name, slot_value)); });
            }
        }
        Ok(deduplicate_overlapping_slots(result))
    }
}

fn are_overlapping(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r1.end > r2.start && r1.start < r2.end
}

fn deduplicate_overlapping_slots(slots: Vec<(String, SlotValue)>) -> Slots {
    let mut deduped: Vec<(String, SlotValue)> = Vec::with_capacity(slots.len());

    for (key, value) in slots {
        if let Some(index) =
            deduped
                .iter()
                .position(|&(_, ref ev)| are_overlapping(&value.range, &ev.range)) {
            fn extract_counts(v: &SlotValue) -> (usize, usize) {
                (tokenize(&v.value).len(), v.value.chars().count())
            }
            let (existing_token_count, existing_char_count) = extract_counts(&deduped[index].1);
            let (token_count, char_count) = extract_counts(&value);

            if token_count > existing_token_count ||
               (token_count == existing_token_count && char_count > existing_char_count) {
                deduped[index] = (key, value);
            } else {
            }
        } else {
            deduped.push((key, value));
        }
    }
    const ESTIMATED_MAX_SLOT: usize = 10;
    const ESTIMATED_MAX_SLOTVALUES: usize = 5;
    deduped
        .into_iter()
        .fold(HashMap::with_capacity(ESTIMATED_MAX_SLOT),
              |mut hm, (slot_name, slot_value)| {
                  hm.entry(slot_name)
                      .or_insert_with(|| Vec::with_capacity(ESTIMATED_MAX_SLOTVALUES))
                      .push(slot_value);
                  hm
              })
}

#[cfg(test)]
mod tests {
    use pipeline::light::configuration::RegexIntentParserConfiguration;
    use super::RegexIntentParser;
    use super::deduplicate_overlapping_slots;
    use pipeline::{IntentParser, IntentClassifierResult, SlotValue};

    fn test_configuration() -> RegexIntentParserConfiguration {
        RegexIntentParserConfiguration {
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
        let parser = RegexIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let intent = parser.get_intent(text, 1.0).unwrap();

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: "dummy_intent_1".to_string(),
            probability: 1.0,
        };

        assert_eq!(intent[0], expected_intent);
    }

    #[test]
    fn test_should_get_entities() {
        // Given
        let parser = RegexIntentParser::new(test_configuration()).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let slots = parser.get_entities(text, "dummy_intent_1").unwrap();

        // Then
        let expected_slots = hashmap![
            "dummy_slot_name".to_string() => vec![SlotValue { value: "dummy_a".to_string(), range: 10..17, entity: "dummy_entity_1".to_string() }],
            "dummy_slot_name2".to_string() => vec![SlotValue { value: "dummy_c".to_string(), range: 37..44, entity: "dummy_entity_2".to_string() }],
        ];
        assert_eq!(slots, expected_slots);
    }

    // TODO test will fail if you inverse lines s3 and s4 (same as in python)
    #[test]
    fn test_should_deduplicate_overlapping_slots() {
        // Given
        let slots = vec![("s1".to_string(),
                          SlotValue {
                              value: "non_overlapping1".to_string(),
                              range: 3..7,
                              entity: "e".to_string(),
                          }),
                         ("s2".to_string(),
                          SlotValue {
                              value: "aaaaaaa".to_string(),
                              range: 9..16,
                              entity: "e1".to_string(),
                          }),
                         ("s3".to_string(),
                          SlotValue {
                              value: "bbbbbbbb".to_string(),
                              range: 10..18,
                              entity: "e1".to_string(),
                          }),
                         ("s4".to_string(),
                          SlotValue {
                              value: "b cccc".to_string(),
                              range: 17..23,
                              entity: "e1".to_string(),
                          }),
                         ("s5".to_string(),
                          SlotValue {
                              value: "non_overlapping2".to_string(),
                              range: 50..60,
                              entity: "e".to_string(),
                          })];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots);

        // Then
        let expected_slots = hashmap![
            "s1".to_string() => vec![SlotValue { value: "non_overlapping1".to_string(), range: 3..7, entity: "e".to_string() }],
            "s4".to_string() => vec![SlotValue { value: "b cccc".to_string(), range: 17..23, entity: "e1".to_string() }],
            "s5".to_string() => vec![SlotValue { value: "non_overlapping2".to_string(), range: 50..60, entity: "e".to_string() }],
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }
}
