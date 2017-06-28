use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;
use rustling_ontology::Lang;

use errors::*;
use core_ontology::{IntentClassifierResult, Slot};
use pipeline::{IntentParser, InternalSlot};
use pipeline::slot_utils::{convert_to_custom_slot, resolve_builtin_slots};
use pipeline::probabilistic::configuration::ProbabilisticParserConfiguration;
use pipeline::probabilistic::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
use pipeline::probabilistic::tagger::{CRFTagger, Tagger};
use pipeline::probabilistic::crf_utils::{tags_to_slots, positive_tagging, replace_builtin_tags};
use utils::miscellaneous::ranges_overlap;
use utils::token::{Token, tokenize};
use builtin_entities::{BuiltinEntityKind, RustlingEntity, RustlingParser};


pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
    taggers: HashMap<String, Box<Tagger>>,
    builtin_entity_parser: Option<Arc<RustlingParser>>
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserConfiguration) -> Result<Self> {
        let taggers: Result<Vec<_>> = config.taggers.into_iter()
            .map(|(intent_name, tagger_config)| Ok((intent_name, Box::new(CRFTagger::new(tagger_config)?) as _)))
            .collect();
        let taggers_map = HashMap::from_iter(taggers?);
        let intent_classifier = Box::new(LogRegIntentClassifier::new(config.intent_classifier)?) as _;
        let builtin_entity_parser = Lang::from_str(&config.language_code).ok()
            .map(|rustling_lang| RustlingParser::get(rustling_lang));

        Ok(ProbabilisticIntentParser {
            intent_classifier,
            slot_name_to_entity_mapping: config.slot_name_to_entity_mapping,
            taggers: taggers_map,
            builtin_entity_parser
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn get_intent(&self, input: &str,
                  intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>> {
        if let Some(intents_set) = intents {
            Ok(
                if intents_set.len() == 1 {
                    Some(
                        IntentClassifierResult {
                            intent_name: intents_set.into_iter().next().unwrap().to_string(),
                            probability: 1.0
                        }
                    )
                } else if let Some(res) = self.intent_classifier.get_intent(input)? {
                    intents_set
                        .get(&res.intent_name)
                        .map(|_| res)
                } else {
                    None
                }
            )
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

        let tags = (*tagger).get_tags(&tokens)?;

        let builtin_slot_names_iter = intent_slots_mapping.iter()
            .filter_map(|(slot_name, entity)|
                BuiltinEntityKind::from_identifier(entity).ok().map(|_| slot_name.to_string())
            );
        let builtin_slot_names = HashSet::from_iter(builtin_slot_names_iter);

        // Remove slots corresponding to builtin entities
        let tagging_scheme = (*tagger).get_tagging_scheme();
        let custom_slots = tags_to_slots(input, &tokens, &tags, tagging_scheme, intent_slots_mapping)
            .into_iter()
            .filter(|s| !builtin_slot_names.contains(&s.slot_name))
            .collect_vec();

        if builtin_slot_names.is_empty() {
            return Ok(custom_slots.into_iter().map(convert_to_custom_slot).collect());
        }

        let updated_tags = replace_builtin_tags(tags, builtin_slot_names);

        let builtin_slots = intent_slots_mapping.iter()
            .filter_map(|(slot_name, entity)|
                BuiltinEntityKind::from_identifier(entity).ok().map(|kind| (slot_name.clone(), kind)))
            .collect_vec();

        let builtin_entity_kinds = builtin_slots.iter()
            .map(|&(_, kind)| kind)
            .unique()
            .collect_vec();

        if let Some(builtin_entity_parser) = self.builtin_entity_parser.as_ref() {
            let builtin_entities = builtin_entity_parser.extract_entities(input, Some(&builtin_entity_kinds));
            let augmented_slots = augment_slots(input,
                                                &tokens,
                                                updated_tags,
                                                &**tagger,
                                                intent_slots_mapping,
                                                builtin_entities,
                                                builtin_slots)?;
            Ok(resolve_builtin_slots(input, augmented_slots, &*builtin_entity_parser))
        } else {
            Ok(custom_slots.into_iter().map(convert_to_custom_slot).collect())
        }
    }
}

fn augment_slots(text: &str,
                 tokens: &[Token],
                 tags: Vec<String>,
                 tagger: &Tagger,
                 intent_slots_mapping: &HashMap<String, String>,
                 builtin_entities: Vec<RustlingEntity>,
                 missing_slots: Vec<(String, BuiltinEntityKind)>) -> Result<Vec<InternalSlot>> {
    let mut augmented_tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    let grouped_entities = builtin_entities.into_iter().group_by(|e| e.entity_kind);

    for (entity_kind, group) in &grouped_entities {
        let spans_ranges = group.map(|e| e.range).collect_vec();
        let tokens_indexes = spans_to_tokens_indexes(&spans_ranges, tokens);
        let related_slots = missing_slots.iter()
            .filter_map(|&(ref slot_name, kind)|
                if kind == entity_kind {
                    Some(slot_name)
                } else {
                    None
                }
            )
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
                let tagging_scheme = tagger.get_tagging_scheme();
                let sub_tags_sequence = positive_tagging(tagging_scheme, slot, indexes.len());
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

    Ok(tags_to_slots(text, tokens, &augmented_tags, tagger.get_tagging_scheme(), &intent_slots_mapping))
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
    use std::result::Result as StdResult;
    use pipeline::probabilistic::crf_utils::TaggingScheme;
    use builtin_entities::{BuiltinEntity, TimeValue, InstantTimeValue, Precision, Grain};

    struct TestTagger {
        tags1: Vec<String>,
        tags2: Vec<String>
    }

    impl Tagger for TestTagger {
        fn get_tags(&self, _: &[Token]) -> Result<Vec<String>> {
            Ok(vec![
                "O".to_string(),
                "O".to_string(),
                "O".to_string(),
                "O".to_string(),
                "O".to_string(),
                "B-location".to_string(),
                "O".to_string(),
                "B-start_date".to_string(),
                "I-start_date".to_string(),
                "I-start_date".to_string(),
                "O".to_string(),
                "O".to_string(),
                "B-end_date".to_string(),
                "O".to_string(),
            ])
        }

        fn get_sequence_probability(&self, _: &[Token], tags: Vec<String>) -> Result<f64> {
            if tags == self.tags1 {
                Ok(0.6)
            } else if tags == self.tags2 {
                Ok(0.4)
            } else {
                Err(format!("Unexpected tags: {:?}", tags).into())
            }
        }

        fn get_tagging_scheme(&self) -> TaggingScheme {
            TaggingScheme::BIO
        }
    }

    struct TestIntentClassifier {
        result: Option<IntentClassifierResult>
    }

    impl IntentClassifier for TestIntentClassifier {
        fn get_intent(&self, _: &str) -> StdResult<Option<IntentClassifierResult>, Error> {
            let res = self.result.clone();
            Ok(res)
        }
    }

    #[test]
    fn augment_slots_works() {
        // Given
        let text = "Find a flight leaving from Paris between today at 9pm and tomorrow at 8am";
        let tokens = tokenize(text);
        let tags = vec![
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "B-location".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
        ];
        let tags1 = vec![
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "B-location".to_string(),
            "O".to_string(),
            "B-start_date".to_string(),
            "I-start_date".to_string(),
            "I-start_date".to_string(),
            "O".to_string(),
            "B-end_date".to_string(),
            "I-end_date".to_string(),
            "I-end_date".to_string(),
        ];
        let tags2 = vec![
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "B-location".to_string(),
            "O".to_string(),
            "B-end_date".to_string(),
            "I-end_date".to_string(),
            "I-end_date".to_string(),
            "O".to_string(),
            "B-start_date".to_string(),
            "I-start_date".to_string(),
            "I-start_date".to_string(),
        ];
        let tagger = TestTagger { tags1, tags2 };
        let intent_slots_mapping = hashmap! {
            "location".to_string() => "location_entity".to_string(),
            "start_date".to_string() => "snips/datetime".to_string(),
            "end_date".to_string() => "snips/datetime".to_string(),
        };
        let start_time = InstantTimeValue {
            value: "today at 9pm".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact
        };
        let end_time = InstantTimeValue {
            value: "tomorrow at 8am".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact
        };
        let builtin_entities = vec![
            RustlingEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: BuiltinEntity::Time(TimeValue::InstantTime(end_time))
            },
            RustlingEntity {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity_kind: BuiltinEntityKind::Time,
                entity: BuiltinEntity::Time(TimeValue::InstantTime(start_time))
            }
        ];
        let missing_slots = vec![("start_date".to_string(), BuiltinEntityKind::Time),
                                 ("end_date".to_string(), BuiltinEntityKind::Time)];

        // When
        let augmented_slots = augment_slots(text,
                                            &*tokens,
                                            tags,
                                            &tagger,
                                            &intent_slots_mapping,
                                            builtin_entities,
                                            missing_slots).unwrap();

        // Then
        let expected_slots: Vec<InternalSlot> = vec![
            InternalSlot {
                value: "Paris".to_string(),
                range: 27..32,
                entity: "location_entity".to_string(),
                slot_name: "location".to_string()
            },
            InternalSlot {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity: "snips/datetime".to_string(),
                slot_name: "start_date".to_string()
            },
            InternalSlot {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity: "snips/datetime".to_string(),
                slot_name: "end_date".to_string()
            }
        ];
        assert_eq!(expected_slots, augmented_slots);
    }

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
            },
            Token {
                value: "def".to_string(),
                char_range: 4..7,
                range: 4..7,
            },
            Token {
                value: "ghi".to_string(),
                char_range: 10..13,
                range: 10..13,
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

    #[test]
    fn replace_tags_works() {
        // Given
        let tags = vec![
            "B-tag1".to_string(),
            "I-tag1".to_string(),
            "O".to_string(),
            "B-start_date".to_string(),
            "B-end_date".to_string(),
            "O".to_string()
        ];
        let builtin_slot_names = hashset! {"start_date".to_string(), "end_date".to_string()};

        // When
        let replaced_tags = replace_builtin_tags(tags, builtin_slot_names);

        // Then
        let expected_tags = vec![
            "B-tag1".to_string(),
            "I-tag1".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string()
        ];
        assert_eq!(expected_tags, replaced_tags);
    }

    #[test]
    fn should_not_return_filtered_out_intent() {
        // Given
        let classifier_result = IntentClassifierResult {
            intent_name: "disabled_intent".to_string(),
            probability: 0.8
        };
        let classifier = TestIntentClassifier { result: Some(classifier_result) };
        let intent_parser = ProbabilisticIntentParser {
            intent_classifier: Box::new(classifier) as _,
            slot_name_to_entity_mapping: hashmap! {},
            taggers: hashmap! {},
            builtin_entity_parser: None
        };
        let text = "hello world";
        let intents_set = Some(hashset! {"allowed_intent1".to_string(), "allowed_intent2".to_string()});

        // When
        let result = intent_parser.get_intent(text, intents_set.as_ref()).unwrap();

        // Then
        assert_eq!(None, result)
    }

    #[test]
    fn should_return_only_allowed_intent() {
        // Given
        let classifier_result = IntentClassifierResult {
            intent_name: "disabled_intent".to_string(),
            probability: 0.8
        };
        let classifier = TestIntentClassifier { result: Some(classifier_result) };
        let intent_parser = ProbabilisticIntentParser {
            intent_classifier: Box::new(classifier) as _,
            slot_name_to_entity_mapping: hashmap! {},
            taggers: hashmap! {},
            builtin_entity_parser: None
        };
        let text = "hello world";
        let intents_set = Some(hashset! {"allowed_intent1".to_string()});

        // When
        let result = intent_parser.get_intent(text, intents_set.as_ref()).unwrap();

        // Then
        let expected_result = Some(IntentClassifierResult { intent_name: "allowed_intent1".to_string(), probability: 1.0 });
        assert_eq!(expected_result, result)
    }
}
