use std::collections::{HashMap, HashSet};
use std::fs;
use std::iter::FromIterator;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use std::sync;

use crfsuite::Tagger as CRFSuiteTagger;
use itertools::Itertools;

use builtin_entity_parsing::{BuiltinEntityParserFactory, CachingBuiltinEntityParser};
use errors::*;
use language::FromLanguage;
use models::{FromPath, SlotFillerModel, SlotFillerModel2};
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::range::ranges_overlap;
use nlu_utils::string::substring_with_char_range;
use nlu_utils::token::{tokenize, Token};
use serde_json;
use slot_filler::crf_utils::*;
use slot_filler::feature_processor::ProbabilisticFeatureProcessor;
use slot_filler::SlotFiller;
use slot_utils::*;
use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind, Language};

pub struct CRFSlotFiller {
    language: Language,
    tagging_scheme: TaggingScheme,
    tagger: sync::Mutex<CRFSuiteTagger>,
    feature_processor: ProbabilisticFeatureProcessor,
    slot_name_mapping: HashMap<String, String>,
    builtin_entity_parser: sync::Arc<CachingBuiltinEntityParser>,
}

impl FromPath for CRFSlotFiller {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let slot_filler_model_path = path.as_ref().join("slot_filler.json");
        let model_file = fs::File::open(slot_filler_model_path)?;
        let model: SlotFillerModel2 = serde_json::from_reader(model_file)?;

        let tagging_scheme = TaggingScheme::from_u8(model.config.tagging_scheme)?;
        let slot_name_mapping = model.slot_name_mapping;
        let feature_processor =
            ProbabilisticFeatureProcessor::new(&model.config.feature_factory_configs)?;
        let crf_path = path.as_ref().join(&model.crf_model_file);
        let tagger = CRFSuiteTagger::create_from_file(crf_path)?;
        let language = Language::from_str(&model.language_code)?;
        let builtin_entity_parser = BuiltinEntityParserFactory::get(language);

        Ok(Self {
            language,
            tagging_scheme,
            tagger: sync::Mutex::new(tagger),
            feature_processor,
            slot_name_mapping,
            builtin_entity_parser,
        })
    }
}

impl CRFSlotFiller {
    pub fn new(config: SlotFillerModel) -> Result<CRFSlotFiller> {
        let tagging_scheme = TaggingScheme::from_u8(config.config.tagging_scheme)?;
        let slot_name_mapping = config.slot_name_mapping;
        let feature_processor =
            ProbabilisticFeatureProcessor::new(&config.config.feature_factory_configs)?;
        let converted_data = ::base64::decode(&config.crf_model_data)?;
        let tagger = CRFSuiteTagger::create_from_memory(converted_data)?;
        let language = Language::from_str(&config.language_code)?;
        let builtin_entity_parser = BuiltinEntityParserFactory::get(language);

        Ok(Self {
            language,
            tagging_scheme,
            tagger: sync::Mutex::new(tagger),
            feature_processor,
            slot_name_mapping,
            builtin_entity_parser,
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

        let augmented_slots = augment_slots(
            text,
            &tokens,
            &updated_tags,
            self,
            &self.slot_name_mapping,
            &self.builtin_entity_parser,
            &builtin_slots,
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

impl CRFSlotFiller {
    pub fn compute_features(&self, text: &str) -> Vec<Vec<(String, String)>> {
        let tokens = tokenize(text, NluUtilsLanguage::from_language(self.language));
        if tokens.is_empty() {
            return vec![];
        };
        self.feature_processor.compute_features(&&*tokens)
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
    builtin_entity_parser: &sync::Arc<CachingBuiltinEntityParser>,
    missing_slots: &[(String, BuiltinEntityKind)],
) -> Result<Vec<InternalSlot>> {
    let builtin_entities = missing_slots
        .iter()
        .map(|&(_, kind)| kind)
        .unique()
        .flat_map(|kind| builtin_entity_parser.extract_entities(text, Some(&[kind]), true))
        .collect();
    let filtered_entities = filter_overlapping_builtins(
        builtin_entities,
        tokens,
        tags,
        slot_filler.get_tagging_scheme(),
    );
    let disambiguated_entities = disambiguate_builtin_entities(filtered_entities);
    let grouped_entities = disambiguated_entities
        .into_iter()
        .fold(HashMap::new(), |mut acc, entity| {
            acc.entry(entity.range.start)
                .or_insert_with(|| vec![])
                .push(entity);
            acc
        })
        .into_iter()
        .map(|(_, entities)| entities)
        .sorted_by_key(|entities| entities[0].range.start);

    let spans_ranges = grouped_entities
        .iter()
        .map(|entities| entities[0].range.clone())
        .collect_vec();
    let tokens_indexes = spans_to_tokens_indexes(&spans_ranges, tokens);
    let slots_permutations = generate_slots_permutations(&*grouped_entities, intent_slots_mapping);

    let mut best_updated_tags = tags.to_vec();
    let mut best_permutation_score: f64 = -1.0;
    for slots in slots_permutations {
        let mut updated_tags = tags.to_vec();
        for (slot_index, slot) in slots.iter().enumerate() {
            let indexes = &tokens_indexes[slot_index];
            let tagging_scheme = slot_filler.get_tagging_scheme();
            let sub_tags_sequence = positive_tagging(tagging_scheme, slot, indexes.len());
            for (index_position, index) in indexes.iter().enumerate() {
                updated_tags[*index] = sub_tags_sequence[index_position].clone();
            }
        }
        let score = slot_filler.get_sequence_probability(tokens, updated_tags.to_vec())?;
        if score > best_permutation_score {
            best_updated_tags = updated_tags;
            best_permutation_score = score;
        }
    }
    let slots = tags_to_slots(
        text,
        tokens,
        &best_updated_tags,
        slot_filler.get_tagging_scheme(),
        intent_slots_mapping,
    )?;
    let filtered_builtin_entities = grouped_entities
        .into_iter()
        .flat_map(|entities| entities)
        .collect();
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

fn disambiguate_builtin_entities(builtin_entities: Vec<BuiltinEntity>) -> Vec<BuiltinEntity> {
    if builtin_entities.is_empty() {
        return builtin_entities;
    }
    let sorted_entities = builtin_entities
        .into_iter()
        .sorted_by_key(|ent| -(ent.range.clone().count() as i8));

    let first_disambiguated_value = sorted_entities[0].clone();

    sorted_entities
        .into_iter()
        .skip(1)
        .fold(vec![first_disambiguated_value], |acc, entity| {
            let mut new_acc = acc.clone();
            let mut conflict = false;
            for disambiguated_entity in acc {
                if ranges_overlap(&entity.range, &disambiguated_entity.range) {
                    conflict = true;
                    if entity.range == disambiguated_entity.range {
                        new_acc.push(entity.clone());
                    }
                    break;
                }
            }
            if !conflict {
                new_acc.push(entity);
            }
            new_acc
        })
        .into_iter()
        .sorted_by_key(|ent| ent.range.start)
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
    use nlu_utils::language::Language as NluUtilsLanguage;
    use snips_nlu_ontology::{Grain, InstantTimeValue, Language, NumberValue, Precision, SlotValue};
    use utils::file_path;

    #[derive(Debug, Fail)]
    pub enum TestError {
        #[fail(display = "Unexpected tags: {:?}", _0)]
        UnknownTags(Vec<String>),
    }

    struct TestSlotFiller {
        tags_list: Vec<Vec<String>>,
        tags_probabilities: Vec<f64>,
    }

    impl TestSlotFiller {
        fn new(tags_slice: &[&[&str]], tags_probabilities: Vec<f64>) -> Self {
            let tags_list = tags_slice
                .iter()
                .map(|tags| tags.iter().map(|t| t.to_string()).collect())
                .collect();
            Self {
                tags_list,
                tags_probabilities,
            }
        }
    }

    impl SlotFiller for TestSlotFiller {
        fn get_tagging_scheme(&self) -> TaggingScheme {
            TaggingScheme::BIO
        }

        fn get_slots(&self, _text: &str) -> Result<Vec<InternalSlot>> {
            Ok(vec![])
        }

        fn get_sequence_probability(&self, _: &[Token], tags: Vec<String>) -> Result<f64> {
            self.tags_list
                .iter()
                .find_position(|t| **t == tags)
                .map(|(i, _)| self.tags_probabilities[i])
                .ok_or(TestError::UnknownTags(tags).into())
        }
    }

    impl FromPath for TestSlotFiller {
        fn from_path<P: AsRef<Path>>(_path: P) -> Result<Self> {
            unimplemented!()
        }
    }

    #[test]
    fn from_path_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("trained_engine")
            .join("probabilistic_intent_parser")
            .join("slot_filler_MakeCoffee");

        // When
        let slot_filler = CRFSlotFiller::from_path(path).unwrap();
        let slots = slot_filler.get_slots("make me two cups of coffee").unwrap();

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "two".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string()
            }
        ];
        assert_eq!(expected_slots, slots);
    }

    #[test]
    fn filter_overlapping_builtin_works() {
        // Given
        let language = NluUtilsLanguage::EN;
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
        let language = NluUtilsLanguage::EN;
        let text = "Find me a flight before 10pm and after 8pm";
        let tokens = tokenize(text, language);

        let tags: Vec<String> = tokens.iter().map(|_| "O".to_string()).collect();

        let tags_list: &[&[&str]] = &[
            &["O", "O", "O", "O", "B-start_date", "I-start_date", "O", "B-end_date", "I-end_date"],
            &["O", "O", "O", "O", "B-end_date", "I-end_date", "O", "B-start_date", "I-start_date"],
            &["O", "O", "O", "O", "O", "O", "O", "O", "O"],
            &["O", "O", "O", "O", "O", "O", "O", "B-start_date", "I-start_date"],
            &["O", "O", "O", "O", "O", "O", "O", "B-end_date", "I-end_date"],
            &["O", "O", "O", "O", "B-start_date", "I-start_date", "O", "O", "O"],
            &["O", "O", "O", "O", "B-end_date", "I-end_date", "O", "O", "O"],
            &["O", "O", "O", "O", "B-start_date", "I-start_date", "O", "B-start_date", "I-start_date"],
            &["O", "O", "O", "O", "B-end_date", "I-end_date", "O", "B-end_date", "I-end_date"],
        ];

        let probabilities = vec![
            0.6,
            0.8,
            0.2,
            0.2,
            0.99,
            0.0,
            0.0,
            0.5,
            0.5
        ];

        let slot_filler = TestSlotFiller::new(tags_list, probabilities);
        let intent_slots_mapping = hashmap! {
            "location".to_string() => "location_entity".to_string(),
            "start_date".to_string() => "snips/datetime".to_string(),
            "end_date".to_string() => "snips/datetime".to_string(),
        };
        let builtin_entity_parser = BuiltinEntityParserFactory::get(Language::EN);
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
            &builtin_entity_parser,
            &missing_slots,
        ).unwrap();

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "after 8pm".to_string(),
                char_range: 33..42,
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

    #[test]
    fn test_should_disambiguate_builtin_entities() {
        // Given
        fn mock_builtin_entity(range: Range<usize>) -> BuiltinEntity {
            BuiltinEntity {
                value: range.clone().map(|i| format!("{}", i)).join(""),
                range,
                entity: SlotValue::Number(NumberValue { value: 0.0 }),
                entity_kind: BuiltinEntityKind::Number,
            }
        }
        let builtin_entities = vec![
            mock_builtin_entity(7..10),
            mock_builtin_entity(9..15),
            mock_builtin_entity(10..17),
            mock_builtin_entity(12..19),
            mock_builtin_entity(9..15),
            mock_builtin_entity(0..5),
            mock_builtin_entity(0..5),
            mock_builtin_entity(0..8),
            mock_builtin_entity(2..5),
            mock_builtin_entity(0..8),
        ];

        // When
        let disambiguated_entities = disambiguate_builtin_entities(builtin_entities);

        // Then
        let expected_entities = vec![
            mock_builtin_entity(0..8),
            mock_builtin_entity(0..8),
            mock_builtin_entity(10..17),
        ];

        assert_eq!(expected_entities, disambiguated_entities);
    }
}
