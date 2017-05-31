use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use errors::*;
use itertools::Itertools;
use pipeline::{IntentClassifierResult, IntentParser, Slot};
use super::intent_classifier::IntentClassifier;
use super::tagger::Tagger;
use super::crf_utils::{tags_to_slots, positive_tagging};
use utils::ranges_overlap;
use preprocessing::{Token, tokenize};
use super::configuration::ProbabilisticParserConfiguration;
use builtin_entities::{EntityKind, RustlingEntity, RustlingParser};
use rustling_ontology::Lang;


pub struct ProbabilisticIntentParser {
    intent_classifier: IntentClassifier,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
    taggers: HashMap<String, Tagger>,
    builtin_entity_parser: Arc<RustlingParser>
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserConfiguration) -> Result<Self> {
        let taggers: Result<Vec<_>> = config.taggers.into_iter()
            .map(|(intent_name, tagger_config)| Ok((intent_name, Tagger::new(tagger_config)?)))
            .collect();
        let taggers_map: HashMap<String, Tagger> = HashMap::from_iter(taggers?);
        let intent_classifier = IntentClassifier::new(config.intent_classifier)?;
        let rustling_lang = Lang::from_str(&config.language_code)?;

        Ok(ProbabilisticIntentParser {
            intent_classifier,
            slot_name_to_entity_mapping: config.slot_name_to_entity_mapping,
            taggers: taggers_map,
            builtin_entity_parser: RustlingParser::get(rustling_lang)
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn get_intent(&self, input: &str,
                  intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>> {
        if let Some(intents_set) = intents {
            if intents_set.len() == 1 {
                Ok(Some(
                    IntentClassifierResult {
                        intent_name: intents_set.into_iter().next().unwrap().to_string(),
                        probability: 1.0
                    }
                ))
            } else {
                let result = self.intent_classifier.get_intent(input)?;
                if let Some(res) = result {
                    if intents_set.contains(&res.intent_name) {
                        Ok(Some(res))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(result)
                }
            }
        } else {
            self.intent_classifier.get_intent(input)
        }
    }

    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>> {
        let tagger = self.taggers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in taggers", intent_name))?;

        let intent_slots_mapping = self.slot_name_to_entity_mapping
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in slots name mapping", intent_name))?;

        let tokens = tokenize(input);
        if tokens.is_empty() {
            return Ok(vec![]);
        }

        let tags = tagger.get_tags(&tokens)?;
        let slots = tags_to_slots(input, &tokens, &tags, tagger.tagging_scheme, intent_slots_mapping)
            .into_iter()
            .filter(|s| EntityKind::from_identifier(&s.entity).is_err())
            .collect();

        let builtin_slots: Vec<String> = intent_slots_mapping.into_iter()
            .filter_map(|(slot_name, entity)|
                EntityKind::from_identifier(entity).ok().map(|_| slot_name.clone()))
            .collect();

        if builtin_slots.is_empty() {
            return Ok(slots);
        }

        let builtin_entity_kinds: Vec<EntityKind> = builtin_slots.iter()
            .map(|s| EntityKind::from_identifier(intent_slots_mapping.get(s).unwrap()).unwrap())
            .collect();

        let builtin_entities = self.builtin_entity_parser.extract_entities(input, Some(&builtin_entity_kinds));

        augment_slots(input, &tokens, tags, tagger, intent_slots_mapping, builtin_entities, builtin_slots)
    }
}

fn augment_slots(text: &str,
                 tokens: &[Token],
                 tags: Vec<String>,
                 tagger: &Tagger,
                 intent_slots_mapping: &HashMap<String, String>,
                 builtin_entities: Vec<RustlingEntity>,
                 missing_slots: Vec<String>) -> Result<Vec<Slot>> {
    let entity_kind_slots_mapping: HashMap<String, EntityKind> = HashMap::from_iter(
        intent_slots_mapping.iter()
            .filter_map(|(slot_name, entity_label)|
                if let Some(kind) = EntityKind::from_identifier(entity_label).ok() {
                    Some((slot_name.clone(), kind))
                } else {
                    None
                })
    );
    let mut augmented_tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    let grouped_entities = builtin_entities.into_iter().group_by(|e| e.kind);

    for (entity_kind, group) in &grouped_entities {
        let spans_ranges = group.map(|e| e.char_range).collect_vec();
        let tokens_indexes = spans_to_tokens_indexes(&spans_ranges, tokens);
        let related_slots = missing_slots.iter()
            .filter(|s| *entity_kind_slots_mapping.get(*s).unwrap() == entity_kind)
            .collect_vec();
        let mut reversed_slots = related_slots.clone();
        reversed_slots.reverse();

        // Hack: we should list all permutations instead of this
        let slots_permutations = vec![related_slots, reversed_slots];

        let mut best_updated_tags = augmented_tags.clone();
        let mut best_permutation_score: f64 = -1.0;
        for slots in slots_permutations.iter() {
            let mut updated_tags = augmented_tags.clone();
            for (slot_index, slot) in slots.iter().enumerate() {
                if slot_index >= tokens_indexes.len() {
                    break
                }
                let ref indexes = tokens_indexes[slot_index];
                let sub_tags_sequence = positive_tagging(tagger.tagging_scheme, slot, indexes.len());
                for (index_position, index) in indexes.iter().enumerate() {
                    updated_tags[*index] = sub_tags_sequence[index_position].clone();
                }
            }
            let score = tagger.get_sequence_probability(tokens, updated_tags.clone())?;
            if score > best_permutation_score {
                best_updated_tags = updated_tags;
                best_permutation_score = score;
            }
        }
        augmented_tags = best_updated_tags;
    }

    Ok(tags_to_slots(text, tokens, &augmented_tags, tagger.tagging_scheme, &intent_slots_mapping))
}

fn spans_to_tokens_indexes(spans: &[Range<usize>], tokens: &[Token]) -> Vec<Vec<usize>> {
    spans.iter()
        .map(|span|
            tokens.iter()
                .enumerate()
                .flat_map(|(i, token)|
                    if ranges_overlap(span, &token.char_range) {
                        Some(i)
                    } else {
                        None
                    })
                .collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_to_tokens_indexes_works() {
        // Given
        let spans = vec![
            0..1,
            2..6,
            5..6,
            9..15
        ];
        let tokens = vec![
            Token {
                value: "abc".to_string(),
                char_range: 0..3,
                range: 0..3,
                entity: None
            },
            Token {
                value: "def".to_string(),
                char_range: 4..7,
                range: 4..7,
                entity: None
            },
            Token {
                value: "ghi".to_string(),
                char_range: 10..13,
                range: 10..13,
                entity: None
            }
        ];

        // When
        let actual_indexes = spans_to_tokens_indexes(&spans, &tokens);

        // Then
        let expected_indexes = vec![
            vec![0],
            vec![0, 1],
            vec![1],
            vec![2],
        ];
        assert_eq!(expected_indexes, actual_indexes);
    }
}
