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
    entity_scopes: HashMap<IntentName, (Vec<BuiltinEntityKind>, Vec<EntityName>)>,
    ignore_stop_words: bool,
    shared_resources: Arc<SharedResources>,
    stop_words_whitelist: HashMap<IntentName, HashSet<String>>,
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
        Ok(DeterministicIntentParser {
            language,
            regexes_per_intent: compile_regexes_per_intent(model.patterns)?,
            group_names_to_slot_names: model.group_names_to_slot_names,
            slot_names_to_entities: model.slot_names_to_entities,
            entity_scopes,
            ignore_stop_words: model.config.ignore_stop_words,
            stop_words_whitelist: model
                .stop_words_whitelist
                .into_iter()
                .map(|(intent, whitelist)| (intent, whitelist.into_iter().collect()))
                .collect(),
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
                .extract_entities(input, Some(builtin_scope.as_ref()), true)?
                .into_iter()
                .map(|entity| entity.into());

            let custom_entities = self
                .shared_resources
                .custom_entity_parser
                .extract_entities(input, Some(custom_scope.as_ref()))?
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
                    self.get_matching_result(
                        input,
                        &*cleaned_formatted_input,
                        regex,
                        intent,
                        Some(&ranges_mapping),
                    )
                    .or_else(|| {
                        self.get_matching_result(input, &*cleaned_input, regex, intent, None)
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
        let stop_words = self.get_intent_stop_words(intent);
        let tokens = tokenize(string, NluUtilsLanguage::from_language(self.language));
        let mut current_idx = 0;
        let mut cleaned_string = "".to_string();
        for mut token in tokens {
            if self.ignore_stop_words && stop_words.contains(&token.normalized_value()) {
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

    fn get_intent_stop_words(&self, intent: &str) -> HashSet<&String> {
        self.stop_words_whitelist
            .get(intent)
            .map(|whitelist| {
                self.shared_resources
                    .stop_words
                    .difference(whitelist)
                    .collect()
            })
            .unwrap_or_else(|| self.shared_resources.stop_words.iter().collect())
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

    fn sample_model() -> DeterministicParserModel {
        DeterministicParserModel {
            language_code: "en".to_string(),
            patterns: hashmap![
                "dummy_intent_1".to_string() => vec![
                    r"^\s*This\s*is\s*a\s*(?P<group1>%DUMMY_ENTITY_1%)\s*query\s*with\s*another\s*(?P<group2>%DUMMY_ENTITY_2%)\s*$".to_string(),
                    r"^\s*(?P<group5>%DUMMY_ENTITY_1%)\s*$".to_string(),
                    r"^\s*This\s*is\s*another\s*(?P<group3>%DUMMY_ENTITY_2%)\s*query\s*$".to_string(),
                    r"^\s*This\s*is\s*another\s*über\s*(?P<group3>%DUMMY_ENTITY_2%)\s*query.\s*$".to_string(),
                    r"^\s*This\s*is\s*another\s*(?P<group4>%DUMMY_ENTITY_2%)?\s*$*".to_string(),
                ],
                "dummy_intent_2".to_string() => vec![
                    r"^\s*This\s*is\s*a\s*(?P<group0>%DUMMY_ENTITY_1%)\s*query\s*from\s*another\s*intent\s*$".to_string()
                ],
                "dummy_intent_3".to_string() => vec![
                    r"^\s*Send\s*(?P<group6>%SNIPSAMOUNTOFMONEY%)\s*to\s*john\s*$".to_string(),
                    r"^\s*Send\s*(?P<group6>%SNIPSAMOUNTOFMONEY%)\s*to\s*john\s*at\s*(?P<group7>%DUMMY_ENTITY_2%)\s*$".to_string()
                ],
                "dummy_intent_4".to_string() => vec![
                    r"^\s*what\s*is\s*(?P<group8>%SNIPSNUMBER%)\s*plus\s*(?P<group8_2>%SNIPSNUMBER%)\s*$".to_string()
                ],
                "dummy_intent_5".to_string() => vec![
                    r"^\s*Send\s*5\s*dollars\s*to\s*(?P<group10>%NAME%)\s*$".to_string(),
                ],
                "dummy_intent_6".to_string() => vec![
                    r"^\s*search\s*$".to_string(),
                    r"^\s*search\s*(?P<group9>%SEARCH_OBJECT%)\s*$".to_string(),
                ],
            ],
            group_names_to_slot_names: hashmap![
                "group0".to_string() => "dummy_slot_name".to_string(),
                "group1".to_string() => "dummy_slot_name".to_string(),
                "group2".to_string() => "dummy_slot_name2".to_string(),
                "group3".to_string() => "dummy_slot_name2".to_string(),
                "group4".to_string() => "dummy_slot_name3".to_string(),
                "group5".to_string() => "dummy_slot_name".to_string(),
                "group6".to_string() => "dummy_slot_name4".to_string(),
                "group7".to_string() => "dummy_slot_name2".to_string(),
                "group8".to_string() => "dummy_slot_name5".to_string(),
                "group9".to_string() => "dummy_slot_name6".to_string(),
                "group10".to_string() => "name".to_string(),
            ],
            slot_names_to_entities: hashmap![
                "dummy_intent_1".to_string() => hashmap![
                    "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                    "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
                    "dummy_slot_name3".to_string() => "dummy_entity_2".to_string(),
                ],
                "dummy_intent_2".to_string() => hashmap![
                    "dummy_slot_name".to_string() => "dummy_entity_1".to_string(),
                ],
                "dummy_intent_3".to_string() => hashmap![
                    "dummy_slot_name2".to_string() => "dummy_entity_2".to_string(),
                    "dummy_slot_name4".to_string() => "snips/amountOfMoney".to_string(),
                ],
                "dummy_intent_4".to_string() => hashmap![
                    "dummy_slot_name5".to_string() => "snips/number".to_string(),
                ],
                "dummy_intent_5".to_string() => hashmap![
                    "name".to_string() => "name".to_string(),
                ],
                "dummy_intent_6".to_string() => hashmap![
                    "dummy_slot_name6".to_string() => "search_object".to_string(),
                ],
            ],
            config: DeterministicParserConfig {
                ignore_stop_words: true,
            },
            stop_words_whitelist: HashMap::new(),
        }
    }

    fn ambiguous_model() -> DeterministicParserModel {
        DeterministicParserModel {
            language_code: "en".to_string(),
            patterns: hashmap![
                "intent_1".to_string() => vec![
                    r"^\s*(?P<group0>%EVENT_TYPE%)\s*(?P<group1>%CLIENT_NAME%)\s*$".to_string(),
                ],
                "intent_2".to_string() => vec![
                    r"^\s*meeting\s*snips\s*$".to_string(),
                ],
            ],
            group_names_to_slot_names: hashmap![
                "group0".to_string() => "event_type".to_string(),
                "group1".to_string() => "client_name".to_string(),
            ],
            slot_names_to_entities: hashmap![
                "intent_1".to_string() => hashmap![
                    "event_type".to_string() => "event_type".to_string(),
                    "client_name".to_string() => "client_name".to_string(),
                ],
                "intent_2".to_string() => hashmap![],
            ],
            config: DeterministicParserConfig {
                ignore_stop_words: true,
            },
            stop_words_whitelist: HashMap::new(),
        }
    }

    #[test]
    fn test_load_from_path() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");

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
        let text = "this is a dummy_a query with another dummy_c";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "dummy_a".to_string(),
                    resolved_value: "dummy_a".to_string(),
                    range: 10..17,
                    entity_identifier: "dummy_entity_1".to_string(),
                },
                CustomEntity {
                    value: "dummy_c".to_string(),
                    resolved_value: "dummy_c".to_string(),
                    range: 37..44,
                    entity_identifier: "dummy_entity_2".to_string(),
                },
            ],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .custom_entity_parser(mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_1".to_string()),
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_ambiguous_intent_should_not_be_parsed() {
        // Given
        let text = "Send 5 dollars to john";
        struct TestBuiltinEntityParser {}

        impl BuiltinEntityParser for TestBuiltinEntityParser {
            fn extract_entities(
                &self,
                _sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
            ) -> Result<Vec<BuiltinEntity>> {
                Ok(
                    if filter_entity_kinds
                        .map(|kinds| kinds.contains(&BuiltinEntityKind::AmountOfMoney))
                        .unwrap_or(true)
                    {
                        vec![BuiltinEntity {
                            value: "5 dollars".to_string(),
                            range: 5..14,
                            entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                                value: 5.,
                                precision: Precision::Exact,
                                unit: Some("dollars".to_string()),
                            }),
                            entity_kind: BuiltinEntityKind::AmountOfMoney,
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
                _sentence: &str,
                filter_entity_kinds: Option<&[String]>,
            ) -> Result<Vec<CustomEntity>> {
                Ok(
                    if filter_entity_kinds
                        .map(|kinds| kinds.contains(&"name".to_string()))
                        .unwrap_or(true)
                    {
                        vec![CustomEntity {
                            value: "john".to_string(),
                            range: 18..22,
                            resolved_value: "john".to_string(),
                            entity_identifier: "name".to_string(),
                        }]
                    } else {
                        vec![]
                    },
                )
            }
        }

        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(TestBuiltinEntityParser {})
            .custom_entity_parser(TestCustomEntityParser {})
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: None,
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_should_disambiguate_intents() {
        // Given
        let text = "meeting snips";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "meeting".to_string(),
                    resolved_value: "meeting".to_string(),
                    range: 0..7,
                    entity_identifier: "event_type".to_string(),
                },
                CustomEntity {
                    value: "snips".to_string(),
                    resolved_value: "snips".to_string(),
                    range: 8..13,
                    entity_identifier: "client_name".to_string(),
                },
            ],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .custom_entity_parser(mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(ambiguous_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intents = parser.get_intents(text).unwrap();

        // Then
        let weight_intent_1 = 1. / 3.;
        let weight_intent_2 = 1.;
        let total_weight = weight_intent_1 + weight_intent_2;
        let expected_intents = vec![
            IntentClassifierResult {
                intent_name: Some("intent_2".to_string()),
                confidence_score: weight_intent_2 / total_weight,
            },
            IntentClassifierResult {
                intent_name: Some("intent_1".to_string()),
                confidence_score: weight_intent_1 / total_weight,
            },
            IntentClassifierResult {
                intent_name: None,
                confidence_score: 0.0,
            },
        ];

        assert_eq!(expected_intents, intents);
    }

    #[test]
    fn test_parse_intent_with_intents_whitelist() {
        // Given
        let text = "this is a dummy_a query with another dummy_c";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "dummy_a".to_string(),
                    resolved_value: "dummy_a".to_string(),
                    range: 10..17,
                    entity_identifier: "dummy_entity_1".to_string(),
                },
                CustomEntity {
                    value: "dummy_c".to_string(),
                    resolved_value: "dummy_c".to_string(),
                    range: 37..44,
                    entity_identifier: "dummy_entity_2".to_string(),
                },
            ],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .custom_entity_parser(mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser
            .parse(text, Some(&["dummy_intent_2"]))
            .unwrap()
            .intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: None,
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_get_intents() {
        // Given
        let text = "Send 5 dollars to john";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![BuiltinEntity {
                value: "5 dollars".to_string(),
                range: 5..14,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 5.,
                    precision: Precision::Exact,
                    unit: Some("dollars".to_string()),
                }),
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let results = parser
            .get_intents(text)
            .unwrap()
            .into_iter()
            .map(|res| {
                let intent_alias = if res.confidence_score > 0. {
                    res.intent_name.unwrap_or_else(|| "null".to_string())
                } else {
                    "unmatched_intent".to_string()
                };
                (intent_alias, res.confidence_score)
            })
            .collect::<Vec<_>>();

        // Then
        let expected_results = vec![
            ("dummy_intent_5".to_string(), 2. / 3.),
            ("dummy_intent_3".to_string(), 1. / 3.),
            ("unmatched_intent".to_string(), 0.0),
            ("unmatched_intent".to_string(), 0.0),
            ("unmatched_intent".to_string(), 0.0),
            ("unmatched_intent".to_string(), 0.0),
            ("unmatched_intent".to_string(), 0.0),
        ];
        assert_eq!(expected_results, results);
    }

    #[test]
    fn test_parse_intent_with_builtin_entity() {
        // Given
        let text = "Send 10 dollars to John";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![BuiltinEntity {
                value: "10 dollars".to_string(),
                range: 5..15,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 10.,
                    precision: Precision::Exact,
                    unit: Some("dollars".to_string()),
                }),
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_3".to_string()),
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_parse_intent_with_entities_from_different_intents() {
        // Given
        let text = "Send 10 dollars to John at the wall";

        #[derive(Default)]
        pub struct MyMockedBuiltinEntityParser;

        impl BuiltinEntityParser for MyMockedBuiltinEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
            ) -> Result<Vec<BuiltinEntity>> {
                let mocked_builtin_entity_number = BuiltinEntity {
                    value: "10".to_string(),
                    range: 5..7,
                    entity: SlotValue::Number(NumberValue { value: 10. }),
                    entity_kind: BuiltinEntityKind::Number,
                };
                let mocked_builtin_entity_money = BuiltinEntity {
                    value: "10 dollars".to_string(),
                    range: 5..15,
                    entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                        value: 10.,
                        precision: Precision::Exact,
                        unit: Some("dollars".to_string()),
                    }),
                    entity_kind: BuiltinEntityKind::AmountOfMoney,
                };
                if sentence != "Send 10 dollars to John at the wall" {
                    return Ok(vec![]);
                }
                Ok(filter_entity_kinds
                    .map(|entity_kinds| {
                        let mut entities = vec![];
                        if entity_kinds.contains(&mocked_builtin_entity_number.entity_kind) {
                            entities.push(mocked_builtin_entity_number.clone())
                        };
                        if entity_kinds.contains(&mocked_builtin_entity_money.entity_kind) {
                            entities.push(mocked_builtin_entity_money.clone())
                        };
                        entities
                    })
                    .unwrap_or_else(|| {
                        vec![mocked_builtin_entity_number, mocked_builtin_entity_money]
                    }))
            }
        }

        #[derive(Default)]
        pub struct MyMockedCustomEntityParser;

        impl CustomEntityParser for MyMockedCustomEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[String]>,
            ) -> Result<Vec<CustomEntity>> {
                let mocked_custom_entity_1 = CustomEntity {
                    value: "John".to_string(),
                    resolved_value: "John".to_string(),
                    range: 19..23,
                    entity_identifier: "dummy_entity_1".to_string(),
                };
                let mocked_custom_entity_2 = CustomEntity {
                    value: "the wall".to_string(),
                    resolved_value: "the wall".to_string(),
                    range: 27..35,
                    entity_identifier: "dummy_entity_2".to_string(),
                };
                if sentence != "Send 10 dollars to John at the wall" {
                    return Ok(vec![]);
                }
                Ok(filter_entity_kinds
                    .map(|entity_kinds| {
                        let mut entities = vec![];
                        if entity_kinds.contains(&mocked_custom_entity_1.entity_identifier) {
                            entities.push(mocked_custom_entity_1.clone())
                        };
                        if entity_kinds.contains(&mocked_custom_entity_2.entity_identifier) {
                            entities.push(mocked_custom_entity_2.clone())
                        };
                        entities
                    })
                    .unwrap_or_else(|| vec![mocked_custom_entity_1, mocked_custom_entity_2]))
            }
        }

        let my_mocked_builtin_entity_parser = MyMockedBuiltinEntityParser {};
        let my_mocked_custom_entity_parser = MyMockedCustomEntityParser {};

        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(my_mocked_builtin_entity_parser)
            .custom_entity_parser(my_mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_3".to_string()),
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_parse_utterance_with_duplicated_slot_name() {
        // Given
        let text = "what is one plus one";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                BuiltinEntity {
                    value: "one".to_string(),
                    range: 8..11,
                    entity: SlotValue::Number(NumberValue { value: 1. }),
                    entity_kind: BuiltinEntityKind::Number,
                },
                BuiltinEntity {
                    value: "one".to_string(),
                    range: 17..20,
                    entity: SlotValue::Number(NumberValue { value: 1. }),
                    entity_kind: BuiltinEntityKind::Number,
                },
            ],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let parsing_result = parser.parse(text, None).unwrap();
        let intent = parsing_result.intent;
        let slots = parsing_result.slots;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_4".to_string()),
            confidence_score: 1.0,
        };
        let expected_slots = vec![
            InternalSlot {
                value: "one".to_string(),
                char_range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "dummy_slot_name5".to_string(),
            },
            InternalSlot {
                value: "one".to_string(),
                char_range: 17..20,
                entity: "snips/number".to_string(),
                slot_name: "dummy_slot_name5".to_string(),
            },
        ];

        assert_eq!(expected_intent, intent);
        assert_eq!(expected_slots, slots);
    }

    #[test]
    fn test_parse_intent_by_ignoring_stop_words() {
        // Given
        let text = "yolo this is a dummy_a query yala with another dummy_c yili";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "dummy_a".to_string(),
                    resolved_value: "dummy_a".to_string(),
                    range: 15..22,
                    entity_identifier: "dummy_entity_1".to_string(),
                },
                CustomEntity {
                    value: "dummy_c".to_string(),
                    resolved_value: "dummy_c".to_string(),
                    range: 47..54,
                    entity_identifier: "dummy_entity_2".to_string(),
                },
            ],
        )]);
        let stop_words = vec!["yolo".to_string(), "yala".to_string(), "yili".to_string()]
            .into_iter()
            .collect();
        let shared_resources = SharedResourcesBuilder::default()
            .custom_entity_parser(mocked_custom_entity_parser)
            .stop_words(stop_words)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_1".to_string()),
            confidence_score: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_parse_slots() {
        // Given
        let text = "this is a dummy_a query with another dummy_c";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "dummy_a".to_string(),
                    resolved_value: "dummy_a".to_string(),
                    range: 10..17,
                    entity_identifier: "dummy_entity_1".to_string(),
                },
                CustomEntity {
                    value: "dummy_c".to_string(),
                    resolved_value: "dummy_c".to_string(),
                    range: 37..44,
                    entity_identifier: "dummy_entity_2".to_string(),
                },
            ],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(sample_model(), shared_resources).unwrap();

        // When
        let slots = parser.parse(text, None).unwrap().slots;

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "dummy_a".to_string(),
                char_range: 10..17,
                entity: "dummy_entity_1".to_string(),
                slot_name: "dummy_slot_name".to_string(),
            },
            InternalSlot {
                value: "dummy_c".to_string(),
                char_range: 37..44,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn test_parse_slots_with_non_ascii_chars() {
        // Given
        let text = "This is another über dummy_cc query!";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "dummy_cc".to_string(),
                resolved_value: "dummy_cc".to_string(),
                range: 21..29,
                entity_identifier: "dummy_entity_2".to_string(),
            }],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(sample_model(), shared_resources).unwrap();

        // When
        let slots = parser.parse(text, None).unwrap().slots;

        // Then
        let expected_slots = vec![InternalSlot {
            value: "dummy_cc".to_string(),
            char_range: 21..29,
            entity: "dummy_entity_2".to_string(),
            slot_name: "dummy_slot_name2".to_string(),
        }];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn test_parse_slots_with_builtin_entity() {
        // Given
        let text = "Send 10 dollars to John at dummy c";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![BuiltinEntity {
                value: "10 dollars".to_string(),
                range: 5..15,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 10.,
                    precision: Precision::Exact,
                    unit: Some("dollars".to_string()),
                }),
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }],
        )]);
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "dummy c".to_string(),
                resolved_value: "dummy c".to_string(),
                range: 27..34,
                entity_identifier: "dummy_entity_2".to_string(),
            }],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .custom_entity_parser(mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let slots = parser.parse(text, None).unwrap().slots;

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "10 dollars".to_string(),
                char_range: 5..15,
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "dummy_slot_name4".to_string(),
            },
            InternalSlot {
                value: "dummy c".to_string(),
                char_range: 27..34,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn test_parse_slots_with_special_tokenized_out_characters() {
        // Given
        let text = "this is another dummy’c";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "dummy’c".to_string(),
                resolved_value: "dummy’c".to_string(),
                range: 16..23,
                entity_identifier: "dummy_entity_2".to_string(),
            }],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );
        let parser = DeterministicIntentParser::new(sample_model(), shared_resources).unwrap();

        // When
        let slots = parser.parse(text, None).unwrap().slots;

        // Then
        let expected_slots = vec![InternalSlot {
            value: "dummy’c".to_string(),
            char_range: 16..23,
            entity: "dummy_entity_2".to_string(),
            slot_name: "dummy_slot_name3".to_string(),
        }];
        assert_eq!(slots, expected_slots);
    }

    #[test]
    fn test_parse_slots_with_stop_word_entity_value() {
        // Given
        let text = "search this please";
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "this".to_string(),
                resolved_value: "this".to_string(),
                range: 7..11,
                entity_identifier: "search_object".to_string(),
            }],
        )]);
        let stop_words = vec!["this".to_string(), "that".to_string(), "please".to_string()]
            .into_iter()
            .collect();
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .stop_words(stop_words)
                .build(),
        );
        let mut parser_model = sample_model();
        parser_model.stop_words_whitelist = hashmap! {
            "dummy_intent_6".to_string() => vec!["this".to_string()].into_iter().collect()
        };
        let parser = DeterministicIntentParser::new(parser_model, shared_resources).unwrap();

        // When
        let result = parser.parse(text, None).unwrap();

        // Then
        let expected_slots = vec![InternalSlot {
            value: "this".to_string(),
            char_range: 7..11,
            entity: "search_object".to_string(),
            slot_name: "dummy_slot_name6".to_string(),
        }];
        let expected_intent = Some("dummy_intent_6".to_string());
        assert_eq!(expected_intent, result.intent.intent_name);
        assert_eq!(expected_slots, result.slots);
    }

    #[test]
    fn test_get_slots() {
        // Given
        let text = "Send 10 dollars to John at dummy c";
        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![BuiltinEntity {
                value: "10 dollars".to_string(),
                range: 5..15,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 10.,
                    precision: Precision::Exact,
                    unit: Some("dollars".to_string()),
                }),
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }],
        )]);
        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![CustomEntity {
                value: "dummy c".to_string(),
                resolved_value: "dummy c".to_string(),
                range: 27..34,
                entity_identifier: "dummy_entity_2".to_string(),
            }],
        )]);
        let shared_resources = SharedResourcesBuilder::default()
            .builtin_entity_parser(mocked_builtin_entity_parser)
            .custom_entity_parser(mocked_custom_entity_parser)
            .build();
        let parser =
            DeterministicIntentParser::new(sample_model(), Arc::new(shared_resources)).unwrap();

        // When
        let slots = parser.get_slots(text, "dummy_intent_3").unwrap();

        // Then
        let expected_slots = vec![
            InternalSlot {
                value: "10 dollars".to_string(),
                char_range: 5..15,
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "dummy_slot_name4".to_string(),
            },
            InternalSlot {
                value: "dummy c".to_string(),
                char_range: 27..34,
                entity: "dummy_entity_2".to_string(),
                slot_name: "dummy_slot_name2".to_string(),
            },
        ];
        assert_eq!(slots, expected_slots);
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
