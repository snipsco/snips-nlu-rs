use std::sync;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::Range;
use std::str::FromStr;

use crfsuite::Tagger as CRFSuiteTagger;
use itertools::Itertools;

use errors::*;
use configurations::SlotFillerConfiguration;
use language::FromLanguage;
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::substring_with_char_range;
use nlu_utils::token::{tokenize, Token};
use slot_filler::crf_utils::*;
use slot_filler::SlotFiller;
use slot_filler::feature_processor::ProbabilisticFeatureProcessor;
use slot_utils::*;
use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind, Language};
use snips_nlu_ontology_parsers::BuiltinEntityParser;

pub struct CRFSlotFiller {
    language: Language,
    tagging_scheme: TaggingScheme,
    tagger: sync::Mutex<CRFSuiteTagger>,
    feature_processor: ProbabilisticFeatureProcessor,
    slot_name_mapping: HashMap<String, String>,
    builtin_entity_parser: sync::Arc<BuiltinEntityParser>,
    exhaustive_permutations_threshold: usize,
}

impl CRFSlotFiller {
    pub fn new(config: SlotFillerConfiguration) -> Result<CRFSlotFiller> {
        let tagging_scheme = TaggingScheme::from_u8(config.config.tagging_scheme)?;
        let slot_name_mapping = config.slot_name_mapping;
        let feature_processor =
            ProbabilisticFeatureProcessor::new(&config.config.feature_factory_configs)?;
        let converted_data = ::base64::decode(&config.crf_model_data)?;
        let tagger = CRFSuiteTagger::create_from_memory(converted_data)?;
        let language = Language::from_str(&config.language_code)?;
        let builtin_entity_parser = BuiltinEntityParser::get(language);

        Ok(Self {
            language,
            tagging_scheme,
            tagger: sync::Mutex::new(tagger),
            feature_processor,
            slot_name_mapping,
            builtin_entity_parser,
            exhaustive_permutations_threshold: config.config.exhaustive_permutations_threshold,
        })
    }
}

impl SlotFiller for CRFSlotFiller {
    fn get_tagging_scheme(&self) -> TaggingScheme {
        self.tagging_scheme
    }

    fn get_slots(&self, text: &str) -> Result<Vec<InternalSlot>> {
        let tokens = tokenize(text, NluUtilsLanguage::from_language(self.language));
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let features = self.feature_processor.compute_features(&&*tokens);
        let tags = self.tagger
            .lock()
            .map_err(|e| format_err!("Poisonous mutex: {}", e))?
            .tag(&features)?
            .into_iter()
            .map(|tag| decode_tag(&*tag))
            .collect::<Result<Vec<String>>>()?;

        let builtin_slot_names_iter = self.slot_name_mapping.iter().filter_map(
            |(slot_name, entity)| {
                BuiltinEntityKind::from_identifier(entity)
                    .ok()
                    .map(|_| slot_name.to_string())
            },
        );
        let slots = tags_to_slots(
            text,
            &tokens,
            &tags,
            self.tagging_scheme,
            &self.slot_name_mapping,
        )?;

        let builtin_slot_names = HashSet::from_iter(builtin_slot_names_iter);

        if builtin_slot_names.is_empty() {
            return Ok(slots);
        }

        let updated_tags = replace_builtin_tags(tags, &builtin_slot_names);

        let builtin_slots = self.slot_name_mapping
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

        let builtin_entities = self.builtin_entity_parser
            .extract_entities(text, Some(&builtin_entity_kinds));
        let augmented_slots = augment_slots(
            text,
            &tokens,
            &updated_tags,
            self,
            &self.slot_name_mapping,
            builtin_entities,
            &builtin_slots,
            self.exhaustive_permutations_threshold,
        )?;
        Ok(augmented_slots)
    }

    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64> {
        let features = self.feature_processor.compute_features(&tokens);
        let tagger = self.tagger
            .lock()
            .map_err(|e| format_err!("poisonous mutex: {}", e))?;
        let tagger_labels = tagger
            .labels()?
            .into_iter()
            .map(|label| decode_tag(&*label))
            .collect::<Result<Vec<String>>>()?;
        let tagger_labels_slice = tagger_labels.iter().map(|l| &**l).collect_vec();
        // Substitute tags that were not seen during training
        let cleaned_tags = tags.into_iter()
            .map(|t| {
                if tagger_labels.contains(&t) {
                    t
                } else {
                    get_substitution_label(&*tagger_labels_slice)
                }
            })
            .map(|t| encode_tag(&*t))
            .collect_vec();
        tagger.set(&features)?;
        Ok(tagger.probability(cleaned_tags)?)
    }
}

// We need to use base64 encoding to ensure ascii encoding because of encoding issues in
// python-crfsuite

fn decode_tag(tag: &str) -> Result<String> {
    let bytes = ::base64::decode(tag)?;
    Ok(String::from_utf8(bytes)?)
}

fn encode_tag(tag: &str) -> String {
    ::base64::encode(tag)
}

fn filter_overlapping_builtins(
    builtin_entities: Vec<BuiltinEntity>,
    tokens: &[Token],
    tags: &[String],
    tagging_scheme: TaggingScheme,
) -> Vec<BuiltinEntity> {
    let slots_ranges = tags_to_slot_ranges(tokens, tags, tagging_scheme);
    builtin_entities
        .into_iter()
        .filter(|ent| {
            !slots_ranges
                .iter()
                .any(|s| ent.range.start < s.char_range.end && ent.range.end > s.char_range.start)
        })
        .collect()
}

fn augment_slots(
    text: &str,
    tokens: &[Token],
    tags: &[String],
    slot_filler: &SlotFiller,
    intent_slots_mapping: &HashMap<String, String>,
    builtin_entities: Vec<BuiltinEntity>,
    missing_slots: &[(String, BuiltinEntityKind)],
    exhaustive_permutations_threshold: usize,
) -> Result<Vec<InternalSlot>> {
    let mut grouped_entities: HashMap<BuiltinEntityKind, Vec<BuiltinEntity>> = HashMap::new();
    let filtered_entities = filter_overlapping_builtins(
        builtin_entities,
        tokens,
        tags,
        slot_filler.get_tagging_scheme(),
    );
    for entity in filtered_entities {
        grouped_entities
            .entry(entity.entity_kind)
            .or_insert_with(|| vec![])
            .push(entity);
    }

    let mut augmented_tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    for (entity_kind, group) in grouped_entities.iter() {
        let spans_ranges = group.into_iter().map(|e| e.range.clone()).collect_vec();
        let num_detected_builtins = spans_ranges.len();
        let tokens_indexes = spans_to_tokens_indexes(&spans_ranges, tokens);
        let related_slots: Vec<&str> = missing_slots
            .iter()
            .filter_map(|&(ref slot_name, kind)| {
                if kind == *entity_kind {
                    let name: &str = &*slot_name;
                    Some(name)
                } else {
                    None
                }
            })
            .collect_vec();

        let slots_permutations = generate_slots_permutations(
            num_detected_builtins,
            related_slots.as_slice(),
            exhaustive_permutations_threshold,
        );
        let mut best_updated_tags = augmented_tags.clone();
        let mut best_permutation_score: f64 = -1.0;
        for slots in &slots_permutations {
            let mut updated_tags = augmented_tags.clone();
            for (slot_index, slot) in slots.iter().enumerate() {
                let indexes = &tokens_indexes[slot_index];
                let tagging_scheme = slot_filler.get_tagging_scheme();
                let sub_tags_sequence = positive_tagging(tagging_scheme, slot, indexes.len());
                for (index_position, index) in indexes.iter().enumerate() {
                    updated_tags[*index] = sub_tags_sequence[index_position].clone();
                }
            }
            let score = slot_filler.get_sequence_probability(tokens, updated_tags.clone())?;
            if score > best_permutation_score {
                best_updated_tags = updated_tags;
                best_permutation_score = score;
            }
        }
        augmented_tags = best_updated_tags;
    }
    let slots = tags_to_slots(
        text,
        tokens,
        &augmented_tags,
        slot_filler.get_tagging_scheme(),
        intent_slots_mapping,
    )?;
    let filtered_builtin_entities = grouped_entities.into_iter().flat_map(|(_, e)| e).collect();
    Ok(reconciliate_builtin_slots(
        text,
        slots,
        filtered_builtin_entities,
    ))
}

fn reconciliate_builtin_slots(
    text: &str,
    slots: Vec<InternalSlot>,
    builtin_entities: Vec<BuiltinEntity>,
) -> Vec<InternalSlot> {
    slots
        .iter()
        .map(|slot| {
            BuiltinEntityKind::from_identifier(&slot.entity)
                .ok()
                .map(|kind| {
                    builtin_entities
                        .iter()
                        .find_position(|builtin_entity| {
                            if builtin_entity.entity_kind != kind {
                                return false;
                            }
                            let be_start = builtin_entity.range.start;
                            let be_end = builtin_entity.range.end;
                            let be_length = be_end - be_start;
                            let slot_start = slot.char_range.start;
                            let slot_end = slot.char_range.end;
                            let slot_length = slot_end - slot_start;
                            be_start <= slot_start && be_end >= slot_end && be_length > slot_length
                        })
                        .map(|(_, be): (_, &BuiltinEntity)| InternalSlot {
                            value: substring_with_char_range(text.to_string(), &be.range),
                            char_range: be.range.clone(),
                            entity: slot.entity.clone(),
                            slot_name: slot.slot_name.clone(),
                        })
                        .unwrap_or(slot.clone())
                })
                .unwrap_or(slot.clone())
        })
        .collect()
}

fn spans_to_tokens_indexes(spans: &[Range<usize>], tokens: &[Token]) -> Vec<Vec<usize>> {
    spans
        .iter()
        .map(|span| {
            tokens
                .iter()
                .enumerate()
                .flat_map(|(i, token)| {
                    if ranges_overlap(span, &token.char_range) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nlu_utils::language::Language;
    use snips_nlu_ontology::{Grain, InstantTimeValue, Precision, SlotValue};

    struct TestSlotFiller {
        tags1: Vec<String>,
        tags2: Vec<String>,
        tags3: Vec<String>,
        tags4: Vec<String>,
        tags5: Vec<String>,
        tags6: Vec<String>,
        tags7: Vec<String>,
    }

    impl SlotFiller for TestSlotFiller {
        fn get_tagging_scheme(&self) -> TaggingScheme {
            TaggingScheme::BIO
        }

        fn get_slots(&self, _text: &str) -> Result<Vec<InternalSlot>> {
            Ok(vec![])
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
                bail!("Unexpected tags: {:?}", tags)
            }
        }
    }

    #[test]
    fn filter_overlapping_builtin_works() {
        // Given
        let language = Language::EN;
        let text =
            "Fïnd ä flïght leävïng from Paris bëtwëen today at 9pm and tomorrow at 8am";
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
            BuiltinEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(end_time.clone()),
            },
            BuiltinEntity {
                value: "today at 9pm".to_string(),
                range: 41..53,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(start_time),
            },
        ];

        // When
        let filtered_entities =
            filter_overlapping_builtins(builtin_entities, &tokens[..], &tags, TaggingScheme::BIO);

        // Then
        let expected_entities = vec![
            BuiltinEntity {
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
        let exhaustive_permutations_threshold = 1;

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

        let slot_filler = TestSlotFiller {
            tags1,
            tags2,
            tags3,
            tags4,
            tags5,
            tags6,
            tags7,
        };
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
            BuiltinEntity {
                value: "tomorrow at 8am".to_string(),
                range: 58..73,
                entity_kind: BuiltinEntityKind::Time,
                entity: SlotValue::InstantTime(end_time),
            },
            BuiltinEntity {
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
            &tags,
            &slot_filler,
            &intent_slots_mapping,
            builtin_entities,
            &missing_slots,
            exhaustive_permutations_threshold,
        ).unwrap();

        // Then
        let expected_slots: Vec<InternalSlot> = vec![
            InternalSlot {
                value: "Paris".to_string(),
                char_range: 27..32,
                entity: "location_entity".to_string(),
                slot_name: "location".to_string(),
            },
            InternalSlot {
                value: "today at 9pm".to_string(),
                char_range: 41..53,
                entity: "snips/datetime".to_string(),
                slot_name: "start_date".to_string(),
            },
            InternalSlot {
                value: "tomorrow at 8am".to_string(),
                char_range: 58..73,
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
    fn test_reconciliate_builtin_slots_works() {
        // Given
        let text = "tomorrow at 8a.m. please";
        let slots = vec![
            InternalSlot {
                value: "tomorrow at 8a.m".to_string(),
                char_range: 0..16,
                entity: BuiltinEntityKind::Time.identifier().to_string(),
                slot_name: "datetime".to_string(),
            },
        ];
        let builtin_entities = vec![
            BuiltinEntity {
                value: "tomorrow at 8a.m.".to_string(),
                range: 0..17,
                entity: SlotValue::InstantTime(InstantTimeValue {
                    value: "2018-03-10 08:00:00 +00:00".to_string(),
                    grain: Grain::Minute,
                    precision: Precision::Exact,
                }),
                entity_kind: BuiltinEntityKind::Time,
            },
        ];

        // When
        let reconciliated_slots = reconciliate_builtin_slots(text, slots, builtin_entities);

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "tomorrow at 8a.m.".to_string(),
                char_range: 0..17,
                entity: BuiltinEntityKind::Time.identifier().to_string(),
                slot_name: "datetime".to_string(),
            },
        ];
        assert_eq!(expected_slots, reconciliated_slots);
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
        let replaced_tags = replace_builtin_tags(tags, &builtin_slot_names);

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
}
