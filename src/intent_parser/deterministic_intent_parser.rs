use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::{format_err, ResultExt};
use log::{debug, info};
use regex::{Regex, RegexBuilder};
use snips_nlu_ontology::{BuiltinEntityKind, IntentClassifierResult, Language};
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_utils::range::ranges_overlap;
use snips_nlu_utils::string::{convert_to_char_range, substring_with_char_range};
use snips_nlu_utils::token::{tokenize, tokenize_light};

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::DeterministicParserModel;
use crate::resources::SharedResources;
use crate::slot_utils::*;
use crate::utils::{
    deduplicate_overlapping_items, replace_entities, EntityName, IntentName, MatchedEntity,
    SlotName,
};

use super::{internal_parsing_result, IntentParser, InternalParsingResult};
use itertools::Itertools;

pub struct DeterministicIntentParser {
    language: Language,
    regexes_per_intent: HashMap<IntentName, Vec<Regex>>,
    group_names_to_slot_names: HashMap<String, SlotName>,
    slot_names_to_entities: HashMap<IntentName, HashMap<SlotName, EntityName>>,
    stop_words: HashSet<String>,
    specific_stop_words: HashMap<IntentName, HashSet<String>>,
    entity_scopes: HashMap<IntentName, (Vec<BuiltinEntityKind>, Vec<EntityName>)>,
    shared_resources: Arc<SharedResources>,
}

impl DeterministicIntentParser {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        info!(
            "Loading deterministic intent parser ({:?}) ...",
            path.as_ref()
        );
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(&parser_model_path).with_context(|_| {
            format!(
                "Cannot open DeterministicIntentParser file '{:?}'",
                &parser_model_path
            )
        })?;
        let model: DeterministicParserModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize DeterministicIntentParser json data")?;
        let parser = Self::new(model, shared_resources);
        info!("Deterministic intent parser loaded");
        parser
    }
}

impl DeterministicIntentParser {
    pub fn new(
        model: DeterministicParserModel,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let language = Language::from_str(&model.language_code)?;
        let entity_scopes = model
            .slot_names_to_entities
            .iter()
            .map(|(intent, mapping)| {
                let builtin_entities = mapping
                    .iter()
                    .flat_map(|(_, entity)| BuiltinEntityKind::from_identifier(entity).ok())
                    .unique()
                    .collect();
                let custom_entities = mapping
                    .iter()
                    .flat_map(|(_, entity)| {
                        if BuiltinEntityKind::from_identifier(entity).is_ok() {
                            None
                        } else {
                            Some(entity.to_string())
                        }
                    })
                    .unique()
                    .collect();
                (intent.to_string(), (builtin_entities, custom_entities))
            })
            .collect();
        let stop_words = if model.config.ignore_stop_words {
            shared_resources.stop_words.clone()
        } else {
            HashSet::new()
        };
        let specific_stop_words = model
            .stop_words_whitelist
            .into_iter()
            .map(|(intent, intent_stop_words)| {
                (
                    intent,
                    stop_words
                        .difference(&intent_stop_words.into_iter().collect())
                        .cloned()
                        .collect(),
                )
            })
            .collect();
        Ok(DeterministicIntentParser {
            language,
            regexes_per_intent: compile_regexes_per_intent(model.patterns)?,
            group_names_to_slot_names: model.group_names_to_slot_names,
            slot_names_to_entities: model.slot_names_to_entities,
            stop_words,
            specific_stop_words,
            entity_scopes,
            shared_resources,
        })
    }
}

impl IntentParser for DeterministicIntentParser {
    fn parse(
        &self,
        input: &str,
        intents_whitelist: Option<&[&str]>,
    ) -> Result<InternalParsingResult> {
        debug!("Extracting intents and slots with deterministic intent parser...");
        let result = self
            .parse_top_intents(input, 1, intents_whitelist)?
            .into_iter()
            .next()
            .and_then(|res| {
                // return None in case of ambiguity
                if res.intent.confidence_score <= 0.5 {
                    None
                } else {
                    Some(res)
                }
            })
            .unwrap_or_else(|| InternalParsingResult {
                intent: IntentClassifierResult {
                    intent_name: None,
                    confidence_score: 1.0,
                },
                slots: vec![],
            });
        debug!("Intent found: '{:?}'", result.intent.intent_name);
        debug!("{} slots extracted", result.slots.len());
        Ok(result)
    }

    fn get_intents(&self, input: &str) -> Result<Vec<IntentClassifierResult>> {
        let nb_intents = self.regexes_per_intent.keys().count();
        let mut top_intents: Vec<IntentClassifierResult> = self
            .parse_top_intents(input, nb_intents, None)?
            .into_iter()
            .map(|res| res.intent)
            .collect();
        let matched_intents: HashSet<String> = top_intents
            .iter()
            .filter_map(|res| res.intent_name.clone())
            .collect();
        for intent in self.regexes_per_intent.keys() {
            if !matched_intents.contains(intent) {
                top_intents.push(IntentClassifierResult {
                    intent_name: Some(intent.to_string()),
                    confidence_score: 0.0,
                });
            }
        }
        // The None intent is not included in the regex patterns and thus it is
        // never matched by the deterministic parser
        top_intents.push(IntentClassifierResult {
            intent_name: None,
            confidence_score: 0.0,
        });
        Ok(top_intents)
    }

    fn get_slots(&self, input: &str, intent: &str) -> Result<Vec<InternalSlot>> {
        if !self.regexes_per_intent.contains_key(intent) {
            return Err(SnipsNluError::UnknownIntent(intent.to_string()).into());
        }
        let filter = vec![intent];
        self.parse(input, Some(&filter)).map(|result| result.slots)
    }
}

impl DeterministicIntentParser {
    #[allow(clippy::map_clone)]
    fn parse_top_intents(
        &self,
        input: &str,
        top_n: usize,
        intents: Option<&[&str]>,
    ) -> Result<Vec<InternalParsingResult>> {
        let mut results = vec![];

        let intents_set: HashSet<&str> = intents
            .map(|intent_list| intent_list.iter().map(|intent| *intent).collect())
            .unwrap_or_else(|| {
                self.slot_names_to_entities
                    .keys()
                    .map(|intent| &**intent)
                    .collect()
            });
        let filtered_entity_scopes = self
            .entity_scopes
            .iter()
            .filter(|(intent, _)| intents_set.contains(&***intent));

        for (intent, (builtin_scope, custom_scope)) in filtered_entity_scopes {
            let builtin_entities = self
                .shared_resources
                .builtin_entity_parser
                .extract_entities(input, Some(builtin_scope.as_ref()), true, 0)?
                .into_iter()
                .map(|entity| entity.into());

            let custom_entities = self
                .shared_resources
                .custom_entity_parser
                .extract_entities(input, Some(custom_scope.as_ref()), 0)?
                .into_iter()
                .map(|entity| entity.into());

            let mut matched_entities: Vec<MatchedEntity> = vec![];
            matched_entities.extend(builtin_entities);
            matched_entities.extend(custom_entities);

            let (ranges_mapping, formatted_input) =
                replace_entities(input, matched_entities, get_entity_placeholder);
            let cleaned_input = self.preprocess_text(input, &**intent);
            let cleaned_formatted_input = self.preprocess_text(&*formatted_input, &**intent);
            if let Some(matching_result_formatted) = self
                .regexes_per_intent
                .get(intent)
                .ok_or_else(|| format_err!("No associated regexes for intent '{}'", intent))?
                .iter()
                .find_map(|regex| {
                    self.get_matching_result(input, &*cleaned_input, regex, intent, None)
                        .or_else(|| {
                            self.get_matching_result(
                                input,
                                &*cleaned_formatted_input,
                                regex,
                                intent,
                                Some(&ranges_mapping),
                            )
                        })
                })
            {
                results.push(matching_result_formatted);
            }
        }

        // In some rare cases there can be multiple ambiguous intents
        // In such cases, priority is given to results containing fewer slots
        let weights = results
            .iter()
            .map(|res| 1. / (1. + res.slots.len() as f32))
            .collect::<Vec<_>>();
        let total_weight: f32 = weights.iter().sum();

        Ok(results
            .into_iter()
            .enumerate()
            .map(|(idx, mut res)| {
                res.intent.confidence_score = weights[idx] / total_weight;
                res
            })
            .sorted_by(|res1, res2| {
                res2.intent
                    .confidence_score
                    .partial_cmp(&res1.intent.confidence_score)
                    .unwrap()
            })
            .take(top_n)
            .collect())
    }

    fn preprocess_text(&self, string: &str, intent: &str) -> String {
        let stop_words = self
            .specific_stop_words
            .get(intent)
            .unwrap_or_else(|| &self.stop_words);
        let tokens = tokenize(string, NluUtilsLanguage::from_language(self.language));
        let mut current_idx = 0;
        let mut cleaned_string = "".to_string();
        for mut token in tokens {
            if stop_words.contains(&token.normalized_value()) {
                token.value = (0..token.value.chars().count()).map(|_| " ").collect();
            }
            let prefix_length = token.char_range.start - current_idx;
            let prefix: String = (0..prefix_length).map(|_| " ").collect();
            cleaned_string = format!("{}{}{}", cleaned_string, prefix, token.value);
            current_idx = token.char_range.end;
        }
        let suffix_length = string.chars().count() - current_idx;
        let suffix: String = (0..suffix_length).map(|_| " ").collect();
        cleaned_string = format!("{}{}", cleaned_string, suffix);
        cleaned_string
    }

    fn get_matching_result(
        &self,
        input: &str,
        formatted_input: &str,
        regex: &Regex,
        intent: &str,
        builtin_entities_ranges_mapping: Option<&HashMap<Range<usize>, Range<usize>>>,
    ) -> Option<InternalParsingResult> {
        if !regex.is_match(formatted_input) {
            return None;
        }

        for caps in regex.captures_iter(&formatted_input) {
            if caps.len() == 0 {
                continue;
            };

            let slots = caps
                .iter()
                .zip(regex.capture_names())
                .skip(1)
                .filter_map(|(opt_match, opt_group_name)| {
                    if let (Some(a_match), Some(group_name)) = (opt_match, opt_group_name) {
                        Some((a_match, group_name))
                    } else {
                        None
                    }
                })
                .map(|(a_match, group_name)| {
                    let group_name = group_name.split('_').collect::<Vec<&str>>()[0];
                    let slot_name = self.group_names_to_slot_names[group_name].to_string();
                    let entity = self.slot_names_to_entities[intent][&slot_name].to_string();
                    let byte_range = a_match.start()..a_match.end();
                    let mut char_range = convert_to_char_range(&formatted_input, &byte_range);
                    if let Some(ranges_mapping) = builtin_entities_ranges_mapping {
                        char_range =
                            ranges_mapping.get(&char_range).cloned().unwrap_or_else(|| {
                                let shift = get_range_shift(&char_range, ranges_mapping);
                                let range_start = (char_range.start as i32 + shift) as usize;
                                let range_end = (char_range.end as i32 + shift) as usize;
                                range_start..range_end
                            });
                    }
                    let value = substring_with_char_range(input.to_string(), &char_range);
                    InternalSlot {
                        value,
                        char_range,
                        entity,
                        slot_name,
                    }
                })
                .collect();
            let deduplicated_slots = deduplicate_overlapping_slots(slots, self.language);
            let result = internal_parsing_result(Some(intent.to_string()), 1.0, deduplicated_slots);
            return Some(result);
        }

        None
    }
}

fn compile_regexes_per_intent(
    patterns: HashMap<IntentName, Vec<String>>,
) -> Result<HashMap<IntentName, Vec<Regex>>> {
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

fn deduplicate_overlapping_slots(
    slots: Vec<InternalSlot>,
    language: Language,
) -> Vec<InternalSlot> {
    let language = NluUtilsLanguage::from_language(language);
    let slots_overlap = |lhs_slot: &InternalSlot, rhs_slot: &InternalSlot| {
        ranges_overlap(&lhs_slot.char_range, &rhs_slot.char_range)
    };
    let slot_sort_key = |slot: &InternalSlot| {
        let tokens_count = tokenize(&slot.value, language).len();
        let chars_count = slot.value.chars().count();
        -((tokens_count + chars_count) as i32)
    };
    let mut deduped = deduplicate_overlapping_items(slots, slots_overlap, slot_sort_key);
    deduped.sort_by_key(|slot| slot.char_range.start);
    deduped
}

fn get_entity_placeholder(entity_label: &str) -> String {
    // Here we don't need language specific tokenization,
    // we just want to generate a feature name, that's why we use EN
    let normalized_entity_label = tokenize_light(entity_label, NluUtilsLanguage::EN)
        .join("")
        .to_uppercase();
    format!("%{}%", normalized_entity_label)
}

fn get_range_shift(
    matched_range: &Range<usize>,
    ranges_mapping: &HashMap<Range<usize>, Range<usize>>,
) -> i32 {
    let mut shift: i32 = 0;
    let mut previous_replaced_range_end: usize = 0;
    let match_start = matched_range.start;
    for (replaced_range, orig_range) in ranges_mapping.iter() {
        if replaced_range.end <= match_start && replaced_range.end > previous_replaced_range_end {
            previous_replaced_range_end = replaced_range.end;
            shift = orig_range.end as i32 - replaced_range.end as i32;
        }
    }
    shift
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::sync::Arc;

    use maplit::hashmap;
    use snips_nlu_ontology::*;

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::models::{DeterministicParserConfig, DeterministicParserModel};
    use crate::resources::loading::load_engine_shared_resources;
    use crate::slot_utils::InternalSlot;
    use crate::testutils::*;

    use super::*;
    use crate::entity_parser::builtin_entity_parser::BuiltinEntityParser;
    use crate::entity_parser::custom_entity_parser::CustomEntityParser;

    fn build_sample_model(
        patterns: HashMap<&str, Vec<&str>>,
        group_names_to_slot_names: HashMap<&str, &str>,
        slot_names_to_entities: HashMap<&str, HashMap<&str, &str>>,
        ignore_stop_words: bool,
        stop_words_whitelist: HashMap<&str, Vec<&str>>,
    ) -> DeterministicParserModel {
        let patterns = patterns
            .into_iter()
            .map(|(intent, intent_patterns)| {
                (
                    intent.to_string(),
                    intent_patterns
                        .into_iter()
                        .map(|pattern| pattern.to_string())
                        .collect(),
                )
            })
            .collect();
        let group_names_to_slot_names = group_names_to_slot_names
            .into_iter()
            .map(|(group_name, slot_name)| (group_name.to_string(), slot_name.to_string()))
            .collect();
        let slot_names_to_entities = slot_names_to_entities
            .into_iter()
            .map(|(intent, slot_name_mapping)| {
                (
                    intent.to_string(),
                    slot_name_mapping
                        .into_iter()
                        .map(|(slot_name, entity)| (slot_name.to_string(), entity.to_string()))
                        .collect(),
                )
            })
            .collect();
        let stop_words_whitelist = stop_words_whitelist
            .into_iter()
            .map(|(intent, stop_words)| {
                (
                    intent.to_string(),
                    stop_words
                        .into_iter()
                        .map(|stop_word| stop_word.to_string())
                        .collect(),
                )
            })
            .collect();
        DeterministicParserModel {
            language_code: "en".to_string(),
            patterns,
            group_names_to_slot_names,
            slot_names_to_entities,
            config: DeterministicParserConfig { ignore_stop_words },
            stop_words_whitelist,
        }
    }

    #[test]
    fn test_load_from_path() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine_beverage");

        let parser_path = trained_engine_path.join("deterministic_intent_parser");

        let shared_resources = load_engine_shared_resources(trained_engine_path).unwrap();
        let intent_parser =
            DeterministicIntentParser::from_path(parser_path, shared_resources).unwrap();

        // When
        let parsing_result = intent_parser.parse("make two cup of coffee", None).unwrap();

        // Then
        let expected_intent = Some("MakeCoffee".to_string());
        let expected_slots = vec![InternalSlot {
            value: "two".to_string(),
            char_range: 5..8,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
        }];
        assert_eq!(expected_intent, parsing_result.intent.intent_name);
        assert_eq!(expected_slots, parsing_result.slots);
    }

    #[test]
    fn test_parse_intent() {
        // Given
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![r"^\s*foo\s*bar\s*baz\s*$"],
                "intent2" => vec![r"^\s*foo\s*bar\s*ban\s*$"],
            ],
            hashmap![],
            hashmap![
                "intent1" => hashmap![],
                "intent2" => hashmap![],
            ],
            false,
            hashmap![],
        );
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("foo bar ban", None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("intent2".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_parse_intent_with_filter() {
        // Given
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![r"^\s*foo\s*bar\s*baz\s*$"],
                "intent2" => vec![r"^\s*foo\s*bar\s*ban\s*$"],
            ],
            hashmap![],
            hashmap![
                "intent1" => hashmap![],
                "intent2" => hashmap![],
            ],
            false,
            hashmap![],
        );
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("foo bar ban", Some(&["intent1"])).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: None,
                confidence_score: 1.0,
            },
            slots: vec![],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_parse_intent_with_stop_words() {
        // Given
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![r"^\s*foo\s*bar\s*baz\s*$"],
                "intent2" => vec![r"^\s*foo\s*bar\s*ban\s*$"],
            ],
            hashmap![],
            hashmap![
                "intent1" => hashmap![],
                "intent2" => hashmap![],
            ],
            true,
            hashmap![],
        );
        let stop_words = vec!["hey".to_string(), "please".to_string()]
            .into_iter()
            .collect();
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .stop_words(stop_words)
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("hey foo bar please ban", None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("intent2".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_parse_utterance_with_duplicated_slot_names() {
        // Given
        let text = "what is one plus one";
        let model = build_sample_model(
            hashmap![
                "math_operation" => vec![
                    "^\\s*what\\s*is\\s*(?P<group0>%SNIPSNUMBER%)\\s*plus\\s*\
                    (?P<group0_2>%SNIPSNUMBER%)\\s*$"
                ],
            ],
            hashmap!["group0" => "number"],
            hashmap![
                "math_operation" => hashmap!["number" => "snips/number"],
            ],
            true,
            hashmap![],
        );
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                BuiltinEntity {
                    value: "one".to_string(),
                    range: 8..11,
                    entity: SlotValue::Number(NumberValue { value: 1. }),
                    alternatives: vec![],
                    entity_kind: BuiltinEntityKind::Number,
                },
                BuiltinEntity {
                    value: "one".to_string(),
                    range: 17..20,
                    entity: SlotValue::Number(NumberValue { value: 1. }),
                    alternatives: vec![],
                    entity_kind: BuiltinEntityKind::Number,
                },
            ],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .builtin_entity_parser(mocked_builtin_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse(text, None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("math_operation".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![
                InternalSlot {
                    value: "one".to_string(),
                    char_range: 8..11,
                    entity: "snips/number".to_string(),
                    slot_name: "number".to_string(),
                },
                InternalSlot {
                    value: "one".to_string(),
                    char_range: 17..20,
                    entity: "snips/number".to_string(),
                    slot_name: "number".to_string(),
                },
            ],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_very_ambiguous_utterances_should_not_be_parsed() {
        // Given
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![r"^\s*(?P<group0>%EVENT%)\s*tomorrow\s*$"],
                "intent2" => vec![r"^\s*call\s(?P<group1>%SNIPSDATETIME%)\s*$"],
            ],
            hashmap!["group0" => "event", "group1" => "time"],
            hashmap![
                "intent1" => hashmap!["event" => "event"],
                "intent2" => hashmap!["time" => "snips/datetime"],
            ],
            true,
            hashmap![],
        );

        struct TestBuiltinEntityParser {}

        impl BuiltinEntityParser for TestBuiltinEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
                _max_alternative_resolved_values: usize,
            ) -> Result<Vec<BuiltinEntity>> {
                if sentence != "call tomorrow" {
                    return Ok(vec![]);
                };
                Ok(
                    if filter_entity_kinds
                        .map(|kinds| kinds.contains(&BuiltinEntityKind::Datetime))
                        .unwrap_or(true)
                    {
                        vec![BuiltinEntity {
                            value: "tomorrow".to_string(),
                            range: 5..13,
                            entity: SlotValue::InstantTime(InstantTimeValue {
                                value: "tomorrow".to_string(),
                                precision: Precision::Exact,
                                grain: Grain::Day,
                            }),
                            alternatives: vec![],
                            entity_kind: BuiltinEntityKind::Datetime,
                        }]
                    } else {
                        vec![]
                    },
                )
            }
        }

        struct TestCustomEntityParser {}

        impl CustomEntityParser for TestCustomEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[String]>,
                _max_alternative_resolved_values: usize,
            ) -> Result<Vec<CustomEntity>> {
                if sentence != "call tomorrow" {
                    return Ok(vec![]);
                };
                Ok(
                    if filter_entity_kinds
                        .map(|kinds| kinds.contains(&"event".to_string()))
                        .unwrap_or(true)
                    {
                        vec![CustomEntity {
                            value: "call".to_string(),
                            alternative_resolved_values: vec![],
                            range: 0..4,
                            resolved_value: "call".to_string(),
                            entity_identifier: "event".to_string(),
                        }]
                    } else {
                        vec![]
                    },
                )
            }
        }
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .builtin_entity_parser(TestBuiltinEntityParser {})
                .custom_entity_parser(TestCustomEntityParser {})
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("call tomorrow", None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: None,
                confidence_score: 1.0,
            },
            slots: vec![],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_slightly_ambiguous_utterances_should_be_parsed() {
        // Given
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![r"^\s*call\s*tomorrow\s*$"],
                "intent2" => vec![r"^\s*call\s(?P<group0>%SNIPSDATETIME%)\s*$"],
            ],
            hashmap!["group0" => "time"],
            hashmap![
                "intent1" => hashmap![],
                "intent2" => hashmap!["time" => "snips/datetime"],
            ],
            true,
            hashmap![],
        );
        struct TestBuiltinEntityParser {}

        impl BuiltinEntityParser for TestBuiltinEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
                _max_alternative_resolved_values: usize,
            ) -> Result<Vec<BuiltinEntity>> {
                if sentence != "call tomorrow" {
                    return Ok(vec![]);
                };
                Ok(
                    if filter_entity_kinds
                        .map(|kinds| kinds.contains(&BuiltinEntityKind::Datetime))
                        .unwrap_or(true)
                    {
                        vec![BuiltinEntity {
                            value: "tomorrow".to_string(),
                            alternatives: vec![],
                            range: 5..13,
                            entity: SlotValue::InstantTime(InstantTimeValue {
                                value: "tomorrow".to_string(),
                                precision: Precision::Exact,
                                grain: Grain::Day,
                            }),
                            entity_kind: BuiltinEntityKind::Datetime,
                        }]
                    } else {
                        vec![]
                    },
                )
            }
        }

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .builtin_entity_parser(TestBuiltinEntityParser {})
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("call tomorrow", None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("intent1".to_string()),
                confidence_score: 2. / 3.,
            },
            slots: vec![],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_parse_slots() {
        // Given
        let text = "meeting with John at Snips either this afternoon or tomorrow";
        let model = build_sample_model(
            hashmap![
                "intent1" => vec![
                    "^\\s*meeting\\s*with\\s*(?P<group0>%NAME%)\\s*at\\s*\
                    (?P<group1>%LOCATION%)\\s*either\\s*(?P<group2>%SNIPSDATETIME%)\\s*\
                    or\\s*(?P<group2_2>%SNIPSDATETIME%)\\s*$"
                ],
            ],
            hashmap!["group0" => "name", "group1" => "location", "group2" => "time"],
            hashmap![
                "intent1" => hashmap![
                    "name" => "name",
                    "location" => "location",
                    "time" => "snips/datetime"
                ],
            ],
            true,
            hashmap![],
        );

        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                BuiltinEntity {
                    value: "this afternoon".to_string(),
                    alternatives: vec![],
                    range: 34..48,
                    entity: SlotValue::InstantTime(InstantTimeValue {
                        value: "this afternoon".to_string(),
                        precision: Precision::Exact,
                        grain: Grain::Hour,
                    }),
                    entity_kind: BuiltinEntityKind::Datetime,
                },
                BuiltinEntity {
                    value: "tomorrow".to_string(),
                    alternatives: vec![],
                    range: 52..60,
                    entity: SlotValue::InstantTime(InstantTimeValue {
                        value: "tomorrow".to_string(),
                        precision: Precision::Exact,
                        grain: Grain::Day,
                    }),
                    entity_kind: BuiltinEntityKind::Datetime,
                },
            ],
        )]);

        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "john".to_string(),
                    resolved_value: "John".to_string(),
                    alternative_resolved_values: vec![],
                    range: 13..17,
                    entity_identifier: "name".to_string(),
                },
                CustomEntity {
                    value: "snips".to_string(),
                    resolved_value: "Snips".to_string(),
                    alternative_resolved_values: vec![],
                    range: 21..26,
                    entity_identifier: "location".to_string(),
                },
            ],
        )]);

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .builtin_entity_parser(mocked_builtin_entity_parser)
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse(text, None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("intent1".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![
                InternalSlot {
                    value: "John".to_string(),
                    char_range: 13..17,
                    entity: "name".to_string(),
                    slot_name: "name".to_string(),
                },
                InternalSlot {
                    value: "Snips".to_string(),
                    char_range: 21..26,
                    entity: "location".to_string(),
                    slot_name: "location".to_string(),
                },
                InternalSlot {
                    value: "this afternoon".to_string(),
                    char_range: 34..48,
                    entity: "snips/datetime".to_string(),
                    slot_name: "time".to_string(),
                },
                InternalSlot {
                    value: "tomorrow".to_string(),
                    char_range: 52..60,
                    entity: "snips/datetime".to_string(),
                    slot_name: "time".to_string(),
                },
            ],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_parse_stop_words_slots() {
        // Given
        let model = build_sample_model(
            hashmap![
                "search" => vec![
                    "^\\s*search\\s*$",
                    "^\\s*search\\s*(?P<group0>%OBJECT%)\\s*$"
                ],
            ],
            hashmap!["group0" => "object"],
            hashmap!["search" => hashmap!["object" => "object"]],
            true,
            hashmap!["search" => vec!["this", "that"]],
        );
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![
            (
                "search this".to_string(),
                vec![CustomEntity {
                    value: "this".to_string(),
                    resolved_value: "this".to_string(),
                    alternative_resolved_values: vec![],
                    range: 7..11,
                    entity_identifier: "object".to_string(),
                }],
            ),
            (
                "search that".to_string(),
                vec![CustomEntity {
                    value: "that".to_string(),
                    resolved_value: "that".to_string(),
                    alternative_resolved_values: vec![],
                    range: 7..11,
                    entity_identifier: "object".to_string(),
                }],
            ),
        ]);

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .stop_words(
                    vec![
                        "the".to_string(),
                        "a".to_string(),
                        "this".to_string(),
                        "that".to_string(),
                    ]
                    .into_iter()
                    .collect(),
                )
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse("search this", None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("search".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![InternalSlot {
                value: "this".to_string(),
                char_range: 7..11,
                entity: "object".to_string(),
                slot_name: "object".to_string(),
            }],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_get_intents() {
        // Given
        let model = build_sample_model(
            hashmap![
                "greeting1" => vec![r"^\s*hello\s*john\s*$"],
                "greeting2" => vec![r"^\s*hello\s*(?P<group0>%NAME%)\s*$"],
                "greeting3" => vec![r"^\s*(?P<group1>%GREETING%)\s*(?P<group0>%NAME%)\s*$"],
            ],
            hashmap!["group0" => "name", "group1" => "greeting"],
            hashmap![
                "greeting1" => hashmap![],
                "greeting2" => hashmap!["name" => "name"],
                "greeting3" => hashmap!["name" => "name", "greeting" => "greeting"],
            ],
            true,
            hashmap![],
        );
        struct TestCustomEntityParser {}

        impl CustomEntityParser for TestCustomEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[String]>,
                _max_alternative_resolved_values: usize,
            ) -> Result<Vec<CustomEntity>> {
                if sentence != "Hello John" {
                    return Ok(vec![]);
                };
                let mut results = vec![];

                if filter_entity_kinds
                    .map(|kinds| kinds.contains(&"greeting".to_string()))
                    .unwrap_or(true)
                {
                    results.push(CustomEntity {
                        value: "Hello".to_string(),
                        range: 0..5,
                        resolved_value: "Hello".to_string(),
                        alternative_resolved_values: vec![],
                        entity_identifier: "greeting".to_string(),
                    });
                };
                if filter_entity_kinds
                    .map(|kinds| kinds.contains(&"name".to_string()))
                    .unwrap_or(true)
                {
                    results.push(CustomEntity {
                        value: "John".to_string(),
                        range: 6..10,
                        resolved_value: "John".to_string(),
                        alternative_resolved_values: vec![],
                        entity_identifier: "name".to_string(),
                    });
                };
                Ok(results)
            }
        }

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(TestCustomEntityParser {})
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let results = parser.get_intents("Hello John").unwrap();

        // Then
        let expected_results = vec![
            IntentClassifierResult {
                intent_name: Some("greeting1".to_string()),
                confidence_score: 1. / (1. + 1. / 2. + 1. / 3.),
            },
            IntentClassifierResult {
                intent_name: Some("greeting2".to_string()),
                confidence_score: (1. / 2.) / (1. + 1. / 2. + 1. / 3.),
            },
            IntentClassifierResult {
                intent_name: Some("greeting3".to_string()),
                confidence_score: (1. / 3.) / (1. + 1. / 2. + 1. / 3.),
            },
            IntentClassifierResult {
                intent_name: None,
                confidence_score: 0.,
            },
        ];

        assert_eq!(expected_results, results);
    }

    #[test]
    fn test_parse_slots_with_non_ascii_chars() {
        // Given
        let text = "Hello über John";

        let model = build_sample_model(
            hashmap!["greeting" => vec![r"^\s*hello\s*über\s*(?P<group0>%NAME%)\s*$"]],
            hashmap!["group0" => "name"],
            hashmap!["greeting" => hashmap!["name" => "name"]],
            true,
            hashmap![],
        );

        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "John".to_string(),
                resolved_value: "John".to_string(),
                alternative_resolved_values: vec![],
                range: 11..15,
                entity_identifier: "name".to_string(),
            }],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let slots = parser.parse(text, None).unwrap().slots;

        // Then
        let expected_slots = vec![InternalSlot {
            value: "John".to_string(),
            char_range: 11..15,
            entity: "name".to_string(),
            slot_name: "name".to_string(),
        }];
        assert_eq!(expected_slots, slots);
    }

    #[test]
    fn test_parse_slots_with_special_tokenized_out_characters() {
        // Given
        let text = "meeting with John O’reilly";
        let model = build_sample_model(
            hashmap!["intent1" => vec![r"^\s*meeting\s*with\s*(?P<group0>%NAME%)\s*$"]],
            hashmap!["group0" => "name"],
            hashmap!["intent1" => hashmap!["name" => "name"]],
            true,
            hashmap![],
        );

        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "John O’reilly".to_string(),
                resolved_value: "John O’reilly".to_string(),
                alternative_resolved_values: vec![],
                range: 13..26,
                entity_identifier: "name".to_string(),
            }],
        )]);

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse(text, None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("intent1".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![InternalSlot {
                value: "John O’reilly".to_string(),
                char_range: 13..26,
                entity: "name".to_string(),
                slot_name: "name".to_string(),
            }],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_get_slots() {
        // Given
        let text = "Hello John";
        let model = build_sample_model(
            hashmap![
                "greeting" => vec![r"^\s*hello\s*(?P<group0>%NAME%)\s*$"],
                "other_intent" => vec![],
            ],
            hashmap!["group0" => "name"],
            hashmap![
                "greeting" => hashmap!["name" => "name"],
                "other_intent" => hashmap![]
            ],
            true,
            hashmap![],
        );

        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "John".to_string(),
                resolved_value: "John".to_string(),
                alternative_resolved_values: vec![],
                range: 6..10,
                entity_identifier: "name".to_string(),
            }],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );

        let parser = DeterministicIntentParser::new(model, shared_resources).unwrap();

        // When
        let slots_1 = parser.get_slots(text, "greeting").unwrap();
        let slots_2 = parser.get_slots(text, "other_intent").unwrap();

        // Then
        let expected_slots_1 = vec![InternalSlot {
            value: "John".to_string(),
            char_range: 6..10,
            entity: "name".to_string(),
            slot_name: "name".to_string(),
        }];

        assert_eq!(expected_slots_1, slots_1);
        assert_eq!(Vec::<InternalSlot>::new(), slots_2);
    }

    #[test]
    fn test_deduplicate_overlapping_slots() {
        // Given
        let language = Language::EN;
        let slots = vec![
            InternalSlot {
                value: "kid".to_string(),
                char_range: 0..3,
                entity: "e1".to_string(),
                slot_name: "s1".to_string(),
            },
            InternalSlot {
                value: "loco".to_string(),
                char_range: 4..8,
                entity: "e1".to_string(),
                slot_name: "s2".to_string(),
            },
            InternalSlot {
                value: "kid loco".to_string(),
                char_range: 0..8,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "song".to_string(),
                char_range: 9..13,
                entity: "e2".to_string(),
                slot_name: "s4".to_string(),
            },
        ];

        // When
        let deduplicated_slots = deduplicate_overlapping_slots(slots, language);

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "kid loco".to_string(),
                char_range: 0..8,
                entity: "e1".to_string(),
                slot_name: "s3".to_string(),
            },
            InternalSlot {
                value: "song".to_string(),
                char_range: 9..13,
                entity: "e2".to_string(),
                slot_name: "s4".to_string(),
            },
        ];
        assert_eq!(deduplicated_slots, expected_slots);
    }

    #[test]
    fn test_replace_entities() {
        // Given
        let text = "the third album of Blink 182 is great";
        let entities = vec![
            MatchedEntity {
                range: 0..9,
                entity_name: BuiltinEntityKind::Ordinal.identifier().to_string(),
            },
            MatchedEntity {
                range: 25..28,
                entity_name: BuiltinEntityKind::Number.identifier().to_string(),
            },
            MatchedEntity {
                range: 19..28,
                entity_name: BuiltinEntityKind::MusicArtist.identifier().to_string(),
            },
        ];

        // When
        let (range_mapping, formatted_text) =
            replace_entities(text, entities, get_entity_placeholder);

        // Then
        let expected_mapping = HashMap::from_iter(vec![(0..14, 0..9), (24..42, 19..28)]);

        let expected_text = "%SNIPSORDINAL% album of %SNIPSMUSICARTIST% is great";
        assert_eq!(expected_mapping, range_mapping);
        assert_eq!(expected_text, &formatted_text);
    }

    #[test]
    fn test_get_builtin_entity_name() {
        // Given
        let entity_label = "snips/datetime";

        // When
        let formatted_label = get_entity_placeholder(entity_label);

        // Then
        assert_eq!("%SNIPSDATETIME%", &formatted_label)
    }

    #[test]
    fn test_get_range_shift() {
        // Given
        let ranges_mapping = hashmap! {
            2..5 => 2..4,
            8..9 => 7..11
        };

        // When / Then
        assert_eq!(-1, get_range_shift(&(6..7), &ranges_mapping));
        assert_eq!(2, get_range_shift(&(12..13), &ranges_mapping));
    }
}
