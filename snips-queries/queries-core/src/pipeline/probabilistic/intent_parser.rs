use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;

use builtin_entities::{BuiltinEntityKind, RustlingEntity, RustlingParser};
use errors::*;
use pipeline::{IntentParser, InternalSlot};
use pipeline::slot_utils::{convert_to_custom_slot, resolve_builtin_slots};
use pipeline::probabilistic::configuration::ProbabilisticParserConfiguration;
use pipeline::probabilistic::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
use pipeline::probabilistic::tagger::{CRFTagger, Tagger};
use pipeline::probabilistic::crf_utils::{
    positive_tagging, replace_builtin_tags, tags_to_slots, tags_to_slot_ranges,
    generate_slots_permutations, TaggingScheme};
use language::LanguageConfig;
use nlu_utils::range::ranges_overlap;
use nlu_utils::token::{Token, tokenize};
use snips_queries_ontology::{IntentClassifierResult, Slot};


pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_name_to_entity_mapping: HashMap<String, HashMap<String, String>>,
    taggers: HashMap<String, Box<Tagger>>,
    builtin_entity_parser: Option<Arc<RustlingParser>>,
    language_config: LanguageConfig,
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserConfiguration) -> Result<Self> {
        let taggers: Result<Vec<_>> = config
            .taggers
            .into_iter()
            .map(|(intent_name, tagger_config)| {
                Ok((intent_name, Box::new(CRFTagger::new(tagger_config)?) as _))
            })
            .collect();
        let taggers_map = HashMap::from_iter(taggers?);
        let intent_classifier =
            Box::new(LogRegIntentClassifier::new(config.intent_classifier)?) as _;
        let language_config = LanguageConfig::from_str(&config.language_code)?;
        let builtin_entity_parser = Some(RustlingParser::get(language_config.to_rust_lang()));

        Ok(ProbabilisticIntentParser {
            intent_classifier,
            slot_name_to_entity_mapping: config.slot_name_to_entity_mapping,
            taggers: taggers_map,
            builtin_entity_parser,
            language_config: language_config,
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn get_intent(
        &self,
        input: &str,
        intents: Option<&HashSet<String>>,
    ) -> Result<Option<IntentClassifierResult>> {
        if let Some(intents_set) = intents {
            Ok(if intents_set.len() == 1 {
                Some(IntentClassifierResult {
                    intent_name: intents_set.into_iter().next().unwrap().to_string(),
                    probability: 1.0,
                })
            } else if let Some(res) = self.intent_classifier.get_intent(input)? {
                intents_set.get(&res.intent_name).map(|_| res)
            } else {
                None
            })
        } else {
            self.intent_classifier.get_intent(input)
        }
    }

    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>> {
        let tagger = self.taggers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in taggers", intent_name))?;

        let intent_slots_mapping = self.slot_name_to_entity_mapping.get(intent_name).ok_or(
            format!("intent {:?} not found in slots name mapping", intent_name),
        )?;

        let tokens = tokenize(input, self.language_config.language);
        if tokens.is_empty() {
            return Ok(vec![]);
        }

        let tags = (*tagger).get_tags(&tokens)?;

        let builtin_slot_names_iter = intent_slots_mapping.iter().filter_map(
            |(slot_name, entity)| {
                BuiltinEntityKind::from_identifier(entity)
                    .ok()
                    .map(|_| slot_name.to_string())
            },
        );
        let builtin_slot_names = HashSet::from_iter(builtin_slot_names_iter);

        // Remove slots corresponding to builtin entities
        let tagging_scheme = (*tagger).get_tagging_scheme();
        let custom_slots =
            tags_to_slots(input, &tokens, &tags, tagging_scheme, intent_slots_mapping)?
                .into_iter()
                .filter(|s| !builtin_slot_names.contains(&s.slot_name))
                .collect_vec();

        if builtin_slot_names.is_empty() {
            return Ok(
                custom_slots
                    .into_iter()
                    .map(convert_to_custom_slot)
                    .collect(),
            );
        }

        let updated_tags = replace_builtin_tags(tags, builtin_slot_names);

        let builtin_slots = intent_slots_mapping
            .iter()
            .filter_map(|(slot_name, entity)| {
                BuiltinEntityKind::from_identifier(entity)
                    .ok()
                    .map(|kind| (slot_name.clone(), kind))
            })
            .collect_vec();

        let builtin_entity_kinds = builtin_slots
            .iter()
            .map(|&(_, kind)| kind)
            .unique()
            .collect_vec();

        if let Some(builtin_entity_parser) = self.builtin_entity_parser.as_ref() {
            let builtin_entities = builtin_entity_parser.extract_entities(input, Some(&builtin_entity_kinds));
            let augmented_slots = augment_slots(
                input,
                &tokens,
                updated_tags,
                &**tagger,
                intent_slots_mapping,
                builtin_entities,
                builtin_slots,
            )?;
            Ok(resolve_builtin_slots(
                input,
                augmented_slots,
                &*builtin_entity_parser,
                Some(&builtin_entity_kinds),
            ))
        } else {
            Ok(
                custom_slots
                    .into_iter()
                    .map(convert_to_custom_slot)
                    .collect(),
            )
        }
    }
}

fn filter_overlapping_builtins(builtin_entities: Vec<RustlingEntity>,
                               tokens: &[Token],
                               tags: &Vec<String>,
                               tagging_scheme: TaggingScheme
) -> Vec<RustlingEntity> {
    let slots_ranges = tags_to_slot_ranges(tokens, tags, tagging_scheme);
    builtin_entities
        .into_iter()
        .filter(|ent| {
            !slots_ranges
                .iter()
                .any(|s| ent.range.start < s.range.end && ent.range.end > s.range.start)
        })
        .collect()
}

fn augment_slots(
    text: &str,
    tokens: &[Token],
    tags: Vec<String>,
    tagger: &Tagger,
    intent_slots_mapping: &HashMap<String, String>,
    builtin_entities: Vec<RustlingEntity>,
    missing_slots: Vec<(String, BuiltinEntityKind)>,
) -> Result<Vec<InternalSlot>> {
    let mut grouped_entities: HashMap<BuiltinEntityKind, Vec<RustlingEntity>> = HashMap::new();
    for entity in filter_overlapping_builtins(builtin_entities, tokens, &tags, tagger.get_tagging_scheme()) {
        grouped_entities.entry(entity.entity_kind).or_insert(vec![]).push(entity);
    }

    let mut augmented_tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    for (entity_kind, group) in grouped_entities {
        let spans_ranges = group.into_iter().map(|e| e.range).collect_vec();
        let num_detected_builtins = spans_ranges.len();
        let tokens_indexes = spans_to_tokens_indexes(&spans_ranges, tokens);
        let related_slots = missing_slots
            .iter()
            .filter_map(|&(ref slot_name, kind)| if kind == entity_kind {
                Some(slot_name)
            } else {
                None
            })
            .collect_vec();

        let slots_permutations = generate_slots_permutations(num_detected_builtins as i32, related_slots);
        let mut best_updated_tags = augmented_tags.clone();
        let mut best_permutation_score: f64 = -1.0;
        for slots in slots_permutations.iter() {
            let mut updated_tags = augmented_tags.clone();
            for (slot_index, slot) in slots.iter().enumerate() {
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

    Ok(tags_to_slots(
        text,
        tokens,
        &augmented_tags,
        tagger.get_tagging_scheme(),
        &intent_slots_mapping,
    )?)
}

fn spans_to_tokens_indexes(spans: &[Range<usize>], tokens: &[Token]) -> Vec<Vec<usize>> {
    spans
        .iter()
        .map(|span| {
            tokens
                .iter()
                .enumerate()
                .flat_map(|(i, token)| if ranges_overlap(span, &token.char_range) {
                    Some(i)
                } else {
                    None
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result::Result as StdResult;
    use nlu_utils::language::Language;
    use snips_queries_ontology::{Grain, InstantTimeValue, IntentClassifierResult, Precision,
                                 SlotValue};
    use pipeline::probabilistic::crf_utils::TaggingScheme;

    struct TestTagger {
        tags1: Vec<String>,
        tags2: Vec<String>,
        tags3: Vec<String>,
        tags4: Vec<String>,
        tags5: Vec<String>,
        tags6: Vec<String>,
        tags7: Vec<String>,
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
                Ok(0.9)
            } else if tags == self.tags2 {
                Ok(0.2)
            } else if tags == self.tags3 {
                Ok(0.4)
            } else if tags == self.tags4 {
                Ok(0.3)
            } else if tags == self.tags5 {
                Ok(0.8)
            } else if tags == self.tags6 {
                Ok(0.26)
            } else if tags == self.tags7 {
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
        result: Option<IntentClassifierResult>,
    }

    impl IntentClassifier for TestIntentClassifier {
        fn get_intent(&self, _: &str) -> StdResult<Option<IntentClassifierResult>, Error> {
            let res = self.result.clone();
            Ok(res)
        }
    }

    #[test]
    fn filter_overlapping_builtin_works() {
        // Given
        let language = Language::EN;
        let text = "Find a flight leaving from Paris between today at 9pm and tomorrow at 8am";
        let tokens = tokenize(text, language);
        let tags = vec![
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "B-location".to_string(),
            "O".to_string(),
            "B-not-a-date".to_string(),
            "I-not-a-date".to_string(),
            "I-not-a-date".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
        ];

        let start_time = InstantTimeValue {
            value: "today at 9pm".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact,
        };
        let end_time = InstantTimeValue {
            value: "tomorrow at 8am".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact,
        };

        let builtin_entities = vec![
            RustlingEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(end_time.clone()),
            },
            RustlingEntity {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(start_time),
            },
        ];

        // When
        let filtered_entities = filter_overlapping_builtins(
            builtin_entities, &tokens[..], &tags, TaggingScheme::BIO);

        // Then
        let expected_entities = vec![
            RustlingEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(end_time),
            },
        ];
        assert_eq!(filtered_entities, expected_entities)
    }

    #[test]
    fn augment_slots_works() {
        // Given
        let language = Language::EN;
        let text = "Find a flight leaving from Paris between today at 9pm and tomorrow at 8am";
        let tokens = tokenize(text, language);
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
        let tags3 = vec![
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
            "B-end_date".to_string(),
            "I-end_date".to_string(),
            "I-end_date".to_string(),
        ];
        let tags4 = vec![
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
            "B-start_date".to_string(),
            "I-start_date".to_string(),
            "I-start_date".to_string(),
        ];
        let tags5 = vec![
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
            "O".to_string(),
            "O".to_string(),
            "O".to_string(),
        ];
        let tags6 = vec![
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
            "O".to_string(),
            "O".to_string(),
        ];
        let tags7 = vec![
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

        let tagger = TestTagger { tags1, tags2, tags3, tags4, tags5, tags6, tags7 };
        let intent_slots_mapping = hashmap! {
            "location".to_string() => "location_entity".to_string(),
            "start_date".to_string() => "snips/datetime".to_string(),
            "end_date".to_string() => "snips/datetime".to_string(),
        };
        let start_time = InstantTimeValue {
            value: "today at 9pm".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact,
        };
        let end_time = InstantTimeValue {
            value: "tomorrow at 8am".to_string(),
            grain: Grain::Hour,
            precision: Precision::Exact,
        };
        let builtin_entities = vec![
            RustlingEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(end_time),
            },
            RustlingEntity {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(start_time),
            },
        ];
        let missing_slots = vec![
            ("start_date".to_string(), BuiltinEntityKind::Time),
            ("end_date".to_string(), BuiltinEntityKind::Time),
        ];

        // When
        let augmented_slots = augment_slots(
            text,
            &*tokens,
            tags,
            &tagger,
            &intent_slots_mapping,
            builtin_entities,
            missing_slots,
        ).unwrap();

        // Then
        let expected_slots: Vec<InternalSlot> = vec![
            InternalSlot {
                value: "Paris".to_string(),
                range: 27..32,
                entity: "location_entity".to_string(),
                slot_name: "location".to_string(),
            },
            InternalSlot {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity: "snips/datetime".to_string(),
                slot_name: "start_date".to_string(),
            },
            InternalSlot {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity: "snips/datetime".to_string(),
                slot_name: "end_date".to_string(),
            },
        ];
        assert_eq!(expected_slots, augmented_slots);
    }

    #[test]
    fn spans_to_tokens_indexes_works() {
        // Given
        let spans = vec![0..1, 2..6, 5..6, 9..15];
        let tokens = vec![
            Token::new("abc".to_string(), 0..3, 0..3),
            Token::new("def".to_string(), 4..7, 4..7),
            Token::new("ghi".to_string(), 10..13, 10..13),
        ];

        // When
        let actual_indexes = spans_to_tokens_indexes(&spans, &tokens);

        // Then
        let expected_indexes = vec![vec![0], vec![0, 1], vec![1], vec![2]];
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
            "O".to_string(),
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
            "O".to_string(),
        ];
        assert_eq!(expected_tags, replaced_tags);
    }

    #[test]
    fn should_not_return_filtered_out_intent() {
        // Given
        let language = Language::EN;
        let language_config = LanguageConfig { language };
        let classifier_result = IntentClassifierResult {
            intent_name: "disabled_intent".to_string(),
            probability: 0.8,
        };
        let classifier = TestIntentClassifier {
            result: Some(classifier_result),
        };
        let intent_parser = ProbabilisticIntentParser {
            intent_classifier: Box::new(classifier) as _,
            slot_name_to_entity_mapping: hashmap! {},
            taggers: hashmap! {},
            builtin_entity_parser: None,
            language_config
        };
        let text = "hello world";
        let intents_set = Some(
            hashset! {"allowed_intent1".to_string(), "allowed_intent2".to_string()},
        );

        // When
        let result = intent_parser
            .get_intent(text, intents_set.as_ref())
            .unwrap();

        // Then
        assert_eq!(None, result)
    }

    #[test]
    fn should_return_only_allowed_intent() {
        // Given
        let language = Language::EN;
        let language_config = LanguageConfig::from_str(&language.to_string()).unwrap();
        let classifier_result = IntentClassifierResult {
            intent_name: "disabled_intent".to_string(),
            probability: 0.8,
        };
        let classifier = TestIntentClassifier {
            result: Some(classifier_result),
        };
        let intent_parser = ProbabilisticIntentParser {
            intent_classifier: Box::new(classifier) as _,
            slot_name_to_entity_mapping: hashmap! {},
            taggers: hashmap! {},
            builtin_entity_parser: None,
            language_config,
        };
        let text = "hello world";
        let intents_set = Some(hashset! {"allowed_intent1".to_string()});

        // When
        let result = intent_parser
            .get_intent(text, intents_set.as_ref())
            .unwrap();

        // Then
        let expected_result = Some(IntentClassifierResult {
            intent_name: "allowed_intent1".to_string(),
            probability: 1.0,
        });
        assert_eq!(expected_result, result)
    }
}

