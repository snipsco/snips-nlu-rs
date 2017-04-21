use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Add;

use itertools::Itertools;
use regex::{RegexSet, RegexSetBuilder};

use errors::*;
use pipeline::{IntentClassifierResult, IntentParser, Slots, SlotValue};

pub struct RegexIntentParser {
    regexes_per_intent: HashMap<String, RegexSet>,
    group_names_to_slot_names: HashMap<String, String>,
    slot_names_to_entities: HashMap<String, String>,
}

impl RegexIntentParser {
    pub fn new(patterns: HashMap<String, Vec<String>>,
               group_names_to_slot_names: HashMap<String, String>,
               slot_names_to_entities: HashMap<String, String>) -> Result<Self> {
        let regexes: Result<_> = patterns
            .into_iter()
            .map(|(intent, patterns)| {
                let mut rb = RegexSetBuilder::new(&patterns);
                rb.case_insensitive(true);
                Ok((intent, rb.build()?))
            })
            .fold_results(hashmap![], |mut h, (intent, regex)| {
                h.insert(intent, regex);
                h
            });

        Ok(RegexIntentParser {
            regexes_per_intent: regexes?,
            group_names_to_slot_names: group_names_to_slot_names,
            slot_names_to_entities: slot_names_to_entities,
        })
    }
}

impl IntentParser for RegexIntentParser {
    fn get_intent(&self,
                  input: &str,
                  probability_threshold: f32,
                  entities: &str) -> Result<Vec<IntentClassifierResult>> {
        let entities_per_intent: Result<_> = self.regexes_per_intent
            .keys()
            .map(|intent_name| Ok((intent_name, self.get_entities(input, intent_name, entities)?)))
            .fold_results(vec![], |mut h, (intent_name, entities)| {
                h.push((intent_name, entities));
                h
            });
        let entities_per_intent = entities_per_intent?;

        let total_nb_entities = entities_per_intent
            .iter()
            .map(|&(_, ref entities)| entities.len())
            .fold(0, Add::add);
        // TODO: handle intents without slots
        if total_nb_entities == 0 {
            bail!("No intent found for given input \"{}\"", input)
        }

        let results = entities_per_intent
            .into_iter()
            .map(|(intent_name, entities)| {
                IntentClassifierResult {
                    name: intent_name.to_string(),
                    probability: entities.len() as f32 / total_nb_entities as f32,
                }
            })
            .filter(|r| r.probability >= probability_threshold)
            .sorted_by(|a, b| {
                a.probability.partial_cmp(&b.probability).unwrap_or(Ordering::Equal).reverse()
            });
        Ok(results)
    }

    fn get_entities(&self,
                    input: &str,
                    intent_name: &str,
                    entities: &str) -> Result<Slots> {
        let regexes = self.regexes_per_intent
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;

        for m in regexes.matches(&input) {
            
        }

        Ok(HashMap::new())
    }
}

fn deduplicate_overlapping_slots(slots: Slots) -> Slots {
    slots
}

#[cfg(test)]
mod tests {
    use super::RegexIntentParser;
    use super::deduplicate_overlapping_slots;
    use pipeline::{IntentParser, IntentClassifierResult, SlotValue};

    #[test]
    fn test_should_get_intent() {
        // Given
        let parser = RegexIntentParser::new(hashmap![], hashmap![], hashmap![]).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let intent = parser.get_intent(text, 1.0, "[]").unwrap();

        // Then
        let expected_intent = IntentClassifierResult {
            name: "dummy_intent_1".to_string(),
            probability: 1.0
        };

        assert_eq!(intent[0], expected_intent);
    }

    #[test]
    fn test_should_get_slots() {
        // Given
        let parser = RegexIntentParser::new(hashmap![], hashmap![], hashmap![]).unwrap();
        let text = "this is a dummy_a query with another dummy_c";

        // When
        let slots = parser.get_entities(text, "dummy_intent_1", "[]").unwrap();

        // Then
        let expected_slots = hashmap![
            "dummy_slot_name".to_string() => vec![SlotValue { value: "dummy_a".to_string(), range: 10..17 }], // dummy_entity_1
            "dummy_slot_name2".to_string() => vec![SlotValue { value: "dummy_c".to_string(), range: 37..44 }], // dummy_entity_2
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn test_should_deduplicate_overlapping_slots() {
        // Given
        let slots = hashmap![
            "s1".to_string() => vec![SlotValue { value: "non_overlapping1".to_string(), range: 3..7 }], // entity: e
            "s2".to_string() => vec![SlotValue { value: "aaaaaaa".to_string(), range: 9..16 }], // entity: e1
            "s3".to_string() => vec![SlotValue { value: "bbbbbbbb".to_string(), range: 10..18 }], // entity: e1
            "s4".to_string() => vec![SlotValue { value: "b cccc".to_string(), range: 17..23 }], // entity: e1
            "s5".to_string() => vec![SlotValue { value: "non_overlapping2".to_string(), range: 50..60 }], // entity: e
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots);

        // Then
        let expected_slots = hashmap![
            "s1".to_string() => vec![SlotValue { value: "non_overlapping1".to_string(), range: 3..7 }], // entity: e
            "s4".to_string() => vec![SlotValue { value: "b cccc".to_string(), range: 17..23 }], // entity: e1
            "s5".to_string() => vec![SlotValue { value: "non_overlapping2".to_string(), range: 50..60 }], // entity: e
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }
}
