use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::ResultExt;
use itertools::Itertools;
use log::debug;
use snips_nlu_ontology::{BuiltinEntityKind, IntentClassifierResult, Language};
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_utils::string::normalize;
use snips_nlu_utils::string::{hash_str_to_i32, substring_with_char_range, suffix_from_char_index};
use snips_nlu_utils::token::tokenize_light;

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::LookupParserModel;
use crate::resources::SharedResources;
use crate::slot_utils::*;
use crate::utils::{deduplicate_overlapping_entities, IntentName, MatchedEntity, SlotName};
use crate::{EntityScope, GroupedEntityScope, InputHash, IntentId, SlotId};

use super::{IntentParser, InternalParsingResult};

/// HashMap based Intent Parser. The normalized/canonical form of an utterance
/// serves as the key and the value is tuple of (intent_id, [vec_of_slots_ids])
///
/// Once a lookup is done at inference, the intent and slots are retrieved by matching
/// their ids to a vec of intent names and a vec of slot names respectively.
pub struct LookupIntentParser {
    language: Language,
    slots_names: Vec<SlotName>,
    intents_names: Vec<IntentName>,
    map: HashMap<InputHash, (IntentId, Vec<SlotId>)>,
    stop_words: HashSet<String>,
    specific_stop_words: HashMap<IntentName, HashSet<String>>,
    entity_scopes: Vec<GroupedEntityScope>,
    shared_resources: Arc<SharedResources>,
}

impl LookupIntentParser {
    /// load parser from the file system
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let parser_model_path = path.as_ref().join("intent_parser.json");
        let model_file = File::open(&parser_model_path).with_context(|_| {
            format!(
                "Cannot open LookupIntentParser file '{:?}'",
                &parser_model_path
            )
        })?;
        let model: LookupParserModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize LookupIntentParser json data")?;
        Self::new(model, shared_resources)
    }
}

impl LookupIntentParser {
    /// create a parser instance
    pub fn new(model: LookupParserModel, shared_resources: Arc<SharedResources>) -> Result<Self> {
        let language = Language::from_str(&model.language_code)?;
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
        Ok(LookupIntentParser {
            language,
            slots_names: model.slots_names,
            intents_names: model.intents_names,
            map: model.map,
            stop_words,
            specific_stop_words,
            entity_scopes: model.entity_scopes,
            shared_resources,
        })
    }
}

impl IntentParser for LookupIntentParser {
    fn parse(
        &self,
        input: &str,
        intents_whitelist: Option<&[&str]>,
    ) -> Result<InternalParsingResult> {
        debug!("Extracting intents and slots with lookup intent parser...");
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
        let nb_intents = self.intents_names.len();
        let mut top_intents: Vec<IntentClassifierResult> = self
            .parse_top_intents(input, nb_intents, None)?
            .into_iter()
            .map(|res| res.intent)
            .collect();
        let matched_intents: HashSet<String> = top_intents
            .iter()
            .filter_map(|res| res.intent_name.clone())
            .collect();
        for intent in self.intents_names.iter() {
            if !matched_intents.contains(intent) {
                top_intents.push(IntentClassifierResult {
                    intent_name: Some(intent.to_string()),
                    confidence_score: 0.0,
                });
            }
        }
        // The None intent is not included in the lookup table and is thus never matched by
        // the lookup parser
        top_intents.push(IntentClassifierResult {
            intent_name: None,
            confidence_score: 0.0,
        });
        Ok(top_intents)
    }

    fn get_slots(&self, input: &str, intent: &str) -> Result<Vec<InternalSlot>> {
        if !self.intents_names.contains(&intent.to_string()) {
            return Err(SnipsNluError::UnknownIntent(intent.to_string()).into());
        }
        let filter = vec![intent];
        self.parse(input, Some(&filter)).map(|result| result.slots)
    }
}

impl LookupIntentParser {
    fn parse_top_intents(
        &self,
        input: &str,
        top_n: usize,
        intents: Option<&[&str]>,
    ) -> Result<Vec<InternalParsingResult>> {
        let mut results_per_intent = HashMap::<String, Vec<InternalParsingResult>>::new();
        for (text_candidate, entities) in self.get_candidates(input, intents)? {
            let candidate_key = hash_str_to_i32(&text_candidate);
            if let Some(result) = self
                .map
                .get(&candidate_key)
                .and_then(|val| self.parse_map_output(input, val, entities, intents))
            {
                if let Some(intent_name) = result.intent.intent_name.as_ref() {
                    results_per_intent
                        .entry(intent_name.to_string())
                        .and_modify(|results| results.push(result.clone()))
                        .or_insert_with(|| vec![result]);
                }
            }
        }
        let results: Vec<(InternalParsingResult, f32)> = results_per_intent
            .into_iter()
            .filter_map(|(_, internal_results)| {
                internal_results
                    .into_iter()
                    .map(|res| {
                        // In some rare cases there can be multiple ambiguous intents
                        // In such cases, priority is given to results containing fewer slots
                        let score = 1. / (1. + res.slots.len() as f32);
                        (res, score)
                    })
                    .max_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap())
            })
            .collect();

        let total_weight: f32 = results.iter().map(|(_, score)| score).sum();

        Ok(results
            .into_iter()
            .map(|(mut res, score)| {
                res.intent.confidence_score = score / total_weight;
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

    fn get_candidates(
        &self,
        input: &str,
        intents_whitelist: Option<&[&str]>,
    ) -> Result<Vec<(String, Vec<MatchedEntity>)>> {
        let mut candidates: Vec<(String, Vec<MatchedEntity>)> = Vec::new();
        for entity_scope in self.entity_scopes.iter() {
            let intent_group: Vec<&String> = entity_scope
                .intent_group
                .iter()
                .filter(|intent| {
                    intents_whitelist
                        .map(|whitelist| whitelist.contains(&intent.as_str()))
                        .unwrap_or(true)
                })
                .collect();
            if intent_group.is_empty() {
                continue;
            }
            let all_entities = self.get_all_entities(input, &entity_scope.entity_scope)?;
            // We generate all subsets of entities to match utterances containing ambivalent
            // words which can be both entity values or random words
            for entities in get_items_combinations(all_entities) {
                let processed_text = replace_entities_with_placeholders(input, entities.as_ref());
                for intent in intent_group.iter() {
                    let cleaned_text = self.preprocess_text(input, intent);
                    let cleaned_processed_text = self.preprocess_text(&processed_text, intent);
                    candidates.push((cleaned_text, vec![]));
                    candidates.push((cleaned_processed_text, entities.clone()));
                }
            }
        }

        Ok(candidates.into_iter().unique().collect())
    }

    fn get_all_entities(
        &self,
        input: &str,
        entity_scope: &EntityScope,
    ) -> Result<Vec<MatchedEntity>> {
        // get builtin entities
        let builtin_scope: Vec<BuiltinEntityKind> = entity_scope
            .builtin
            .iter()
            .map(|identifier| BuiltinEntityKind::from_identifier(identifier))
            .collect::<Result<Vec<_>>>()?;
        let builtin_entities = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(input, Some(builtin_scope.as_ref()), true)?
            .into_iter()
            .map(|entity| entity.into());
        // get custom entities
        let custom_entities = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(input, Some(entity_scope.custom.as_ref()))?
            .into_iter()
            .map(|entity| entity.into());

        // combine entities
        let mut all_entities: Vec<MatchedEntity> = Vec::new();
        all_entities.extend(builtin_entities);
        all_entities.extend(custom_entities);

        Ok(deduplicate_overlapping_entities(all_entities))
    }

    fn parse_map_output(
        &self,
        input: &str,
        output: &(IntentId, Vec<SlotId>),
        entities: Vec<MatchedEntity>,
        intents: Option<&[&str]>,
    ) -> Option<InternalParsingResult> {
        let (intent_id, slots_ids) = output;
        // assert invariant: length of slot ids matches that of entities
        debug_assert!(slots_ids.len() == entities.len());
        let intent_name = &self.intents_names[*intent_id as usize];
        if intents
            .map(|intents_list| intents_list.contains(&intent_name.as_ref()))
            .unwrap_or(true)
        {
            let intent = IntentClassifierResult {
                intent_name: Some(intent_name.to_string()),
                confidence_score: 1.0,
            };
            // get slots and return result
            // we assume entities are sorted by their ranges
            let mut slots = vec![];
            for (slot_id, entity) in slots_ids.iter().zip(entities.iter()) {
                let slot_name = &self.slots_names[*slot_id as usize];
                let entity_name = &entity.entity_name;
                let char_range = &entity.range;
                let value = substring_with_char_range(input.to_string(), &char_range);
                slots.push(InternalSlot {
                    value,
                    char_range: char_range.clone(),
                    entity: entity_name.to_string(),
                    slot_name: slot_name.to_string(),
                });
            }
            Some(InternalParsingResult { intent, slots })
        } else {
            // if intent name not in intents, return None
            None
        }
    }
}

impl LookupIntentParser {
    fn preprocess_text(&self, string: &str, intent: &str) -> String {
        let stop_words = self
            .specific_stop_words
            .get(intent)
            .unwrap_or_else(|| &self.stop_words);
        tokenize_light(string, NluUtilsLanguage::from_language(self.language))
            .into_iter()
            .filter(|tkn| !stop_words.contains(&normalize(tkn)))
            .collect::<Vec<String>>()
            .join(" ")
            .to_lowercase()
    }
}

fn get_entity_placeholder(entity_label: &str) -> String {
    // Here we don't need language specific tokenization,
    // we just want to generate a feature name, that's why we use EN
    let normalized_entity_label = tokenize_light(entity_label, NluUtilsLanguage::EN)
        .join("")
        .to_uppercase();
    format!("%{}%", normalized_entity_label)
}

fn replace_entities_with_placeholders(text: &str, entities: &[MatchedEntity]) -> String {
    if entities.is_empty() {
        text.to_string()
    } else {
        let mut processed_text = String::new();
        let mut cur_idx = 0;
        for entity in entities.iter() {
            let start = entity.range.start as usize;
            let end = entity.range.end as usize;
            let prefix_txt = substring_with_char_range(text.to_string(), &(cur_idx..start));
            let place_holder = get_entity_placeholder(&entity.entity_name);
            processed_text.push_str(&prefix_txt);
            processed_text.push_str(&place_holder);
            cur_idx = end
        }
        processed_text.push_str(&suffix_from_char_index(text.to_string(), cur_idx));
        processed_text
    }
}

fn get_items_combinations<T>(items: Vec<T>) -> Vec<Vec<T>>
where
    T: Clone,
{
    let mut combinations: Vec<Vec<T>> = (1..items.len() + 1)
        .rev()
        .flat_map(|nb_entities| items.clone().into_iter().combinations(nb_entities))
        .collect();
    combinations.insert(0, vec![]);
    combinations
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use std::iter::FromIterator;
    use std::sync::Arc;

    use maplit::hashmap;
    use snips_nlu_ontology::*;

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::models::{LookupParserConfig, LookupParserModel};
    use crate::resources::loading::load_engine_shared_resources;
    use crate::slot_utils::InternalSlot;
    use crate::testutils::*;

    use super::*;
    use crate::entity_parser::{BuiltinEntityParser, CustomEntityParser};

    fn build_sample_model<T>(
        slots_names: Vec<T>,
        intents_names: Vec<T>,
        map: HashMap<InputHash, (IntentId, Vec<SlotId>)>,
        entity_scopes: Vec<GroupedEntityScope>,
        stop_words_whitelist: HashMap<String, Vec<String>>,
        ignore_stop_words: bool,
    ) -> LookupParserModel
    where
        T: Into<String>,
    {
        let slots_names = slots_names.into_iter().map(|name| name.into()).collect();
        let intents_names = intents_names.into_iter().map(|name| name.into()).collect();
        LookupParserModel {
            language_code: "en".to_string(),
            slots_names,
            intents_names,
            map,
            entity_scopes,
            stop_words_whitelist,
            config: LookupParserConfig { ignore_stop_words },
        }
    }

    #[test]
    fn test_load_from_path() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");

        let parser_path = trained_engine_path.join("lookup_intent_parser");

        let shared_resources = load_engine_shared_resources(trained_engine_path).unwrap();
        let intent_parser = LookupIntentParser::from_path(parser_path, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("foo bar baz") => (0, vec![]),
            hash_str_to_i32("foo bar ban") => (1, vec![]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["intent1".to_string(), "intent2".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec![],
            },
        }];
        let model = build_sample_model(
            vec![],
            vec!["intent1", "intent2"],
            map,
            entity_scopes,
            hashmap![],
            false,
        );
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("foo bar baz") => (0, vec![]),
            hash_str_to_i32("foo bar ban") => (1, vec![]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["intent1".to_string(), "intent2".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec![],
            },
        }];
        let model = build_sample_model(
            vec![],
            vec!["intent1", "intent2"],
            map,
            entity_scopes,
            hashmap![],
            false,
        );
        let shared_resources = Arc::new(SharedResourcesBuilder::default().build());
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("foo bar baz") => (0, vec![]),
            hash_str_to_i32("foo bar ban") => (1, vec![]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["intent1".to_string(), "intent2".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec![],
            },
        }];
        let model = build_sample_model(
            vec![],
            vec!["intent1", "intent2"],
            map,
            entity_scopes,
            hashmap![],
            true,
        );
        let stop_words = vec!["hey".to_string(), "please".to_string()]
            .into_iter()
            .collect();
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .stop_words(stop_words)
                .build(),
        );
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
    fn test_parse_intent_with_duplicated_slot_names() {
        // Given
        let text = "what is one plus one";
        let map = hashmap![
            hash_str_to_i32("what is % snipsnumber % plus % snipsnumber %") => (0, vec![0, 0]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["math_operation".to_string()],
            entity_scope: EntityScope {
                builtin: vec!["snips/number".to_string()],
                custom: vec![],
            },
        }];
        let model = build_sample_model(
            vec!["number"],
            vec!["math_operation"],
            map,
            entity_scopes,
            hashmap![],
            false,
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
    fn test_parse_intent_with_ambivalent_words() {
        // Given
        let text = "give a daisy to emily";
        let map = hashmap![
            hash_str_to_i32("give a rose to % name %") => (0, vec![0]),
            hash_str_to_i32("give a daisy to % name %") => (0, vec![0]),
            hash_str_to_i32("give a tulip to % name %") => (0, vec![0]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["give_flower".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec!["name".to_string()],
            },
        }];
        let model = build_sample_model(
            vec!["name"],
            vec!["give_flower"],
            map,
            entity_scopes,
            hashmap![],
            false,
        );

        let mocked_custom_entity_parser = MockedCustomEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                CustomEntity {
                    value: "daisy".to_string(),
                    resolved_value: "daisy".to_string(),
                    alternative_resolved_values: vec![],
                    range: 7..12,
                    entity_identifier: "name".to_string(),
                },
                CustomEntity {
                    value: "emily".to_string(),
                    resolved_value: "emily".to_string(),
                    alternative_resolved_values: vec![],
                    range: 16..21,
                    entity_identifier: "name".to_string(),
                },
            ],
        )]);
        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .custom_entity_parser(mocked_custom_entity_parser)
                .build(),
        );

        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

        // When
        let parsing = parser.parse(text, None).unwrap();

        // Then
        let expected_parsing = InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: Some("give_flower".to_string()),
                confidence_score: 1.0,
            },
            slots: vec![InternalSlot {
                value: "emily".to_string(),
                char_range: 16..21,
                entity: "name".to_string(),
                slot_name: "name".to_string(),
            }],
        };

        assert_eq!(expected_parsing, parsing);
    }

    #[test]
    fn test_very_ambiguous_utterances_should_not_be_parsed() {
        // Given
        let map = hashmap![
            hash_str_to_i32("% event % tomorrow") => (0, vec![0]),
            hash_str_to_i32("call % snipsdatetime %") => (1, vec![1]),
        ];
        let entity_scopes = vec![
            GroupedEntityScope {
                intent_group: vec!["intent1".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec!["event".to_string()],
                },
            },
            GroupedEntityScope {
                intent_group: vec!["intent2".to_string()],
                entity_scope: EntityScope {
                    builtin: vec!["snips/datetime".to_string()],
                    custom: vec![],
                },
            },
        ];
        let model = build_sample_model(
            vec!["event", "time"],
            vec!["intent1", "intent2"],
            map,
            entity_scopes,
            hashmap![],
            true,
        );

        struct TestBuiltinEntityParser {}

        impl BuiltinEntityParser for TestBuiltinEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
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
                            range: 0..4,
                            resolved_value: "call".to_string(),
                            alternative_resolved_values: vec![],
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("call tomorrow") => (0, vec![]),
            hash_str_to_i32("call % snipsdatetime %") => (1, vec![0]),
        ];
        let entity_scopes = vec![
            GroupedEntityScope {
                intent_group: vec!["intent1".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec![],
                },
            },
            GroupedEntityScope {
                intent_group: vec!["intent2".to_string()],
                entity_scope: EntityScope {
                    builtin: vec!["snips/datetime".to_string()],
                    custom: vec![],
                },
            },
        ];
        let model = build_sample_model(
            vec!["time"],
            vec!["intent1", "intent2"],
            map,
            entity_scopes,
            hashmap![],
            true,
        );

        struct TestBuiltinEntityParser {}

        impl BuiltinEntityParser for TestBuiltinEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[BuiltinEntityKind]>,
                _use_cache: bool,
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

        let shared_resources = Arc::new(
            SharedResourcesBuilder::default()
                .builtin_entity_parser(TestBuiltinEntityParser {})
                .build(),
        );
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
        hash_str_to_i32("meeting with % name % at % location % either \
        % snipsdatetime % or % snipsdatetime %") => (0, vec![0, 1, 2, 2]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["intent1".to_string()],
            entity_scope: EntityScope {
                builtin: vec!["snips/datetime".to_string()],
                custom: vec!["name".to_string(), "location".to_string()],
            },
        }];
        let model = build_sample_model(
            vec!["name", "location", "time"],
            vec!["intent1"],
            map,
            entity_scopes,
            hashmap![],
            true,
        );

        let mocked_builtin_entity_parser = MockedBuiltinEntityParser::from_iter(vec![(
            text.to_string(),
            vec![
                BuiltinEntity {
                    value: "this afternoon".to_string(),
                    range: 34..48,
                    entity: SlotValue::InstantTime(InstantTimeValue {
                        value: "this afternoon".to_string(),
                        precision: Precision::Exact,
                        grain: Grain::Hour,
                    }),
                    alternatives: vec![],
                    entity_kind: BuiltinEntityKind::Datetime,
                },
                BuiltinEntity {
                    value: "tomorrow".to_string(),
                    range: 52..60,
                    entity: SlotValue::InstantTime(InstantTimeValue {
                        value: "tomorrow".to_string(),
                        precision: Precision::Exact,
                        grain: Grain::Day,
                    }),
                    alternatives: vec![],
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("search") => (0, vec![]),
            hash_str_to_i32("search % object %") => (0, vec![0]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["search".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec!["object".to_string()],
            },
        }];
        let model = build_sample_model(
            vec!["object"],
            vec!["search"],
            map,
            entity_scopes,
            hashmap!["search".to_string() => vec!["this".to_string(), "that".to_string()]],
            true,
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("hello john") => (0, vec![]),
            hash_str_to_i32("hello % name %") => (1, vec![0]),
            hash_str_to_i32("% greeting % % name %") => (2, vec![1, 0]),
        ];
        let entity_scopes = vec![
            GroupedEntityScope {
                intent_group: vec!["greeting1".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec![],
                },
            },
            GroupedEntityScope {
                intent_group: vec!["greeting2".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec!["name".to_string()],
                },
            },
            GroupedEntityScope {
                intent_group: vec!["greeting3".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec!["name".to_string(), "greeting".to_string()],
                },
            },
        ];
        let model = build_sample_model(
            vec!["name", "greeting"],
            vec!["greeting1", "greeting2", "greeting3"],
            map,
            entity_scopes,
            hashmap![],
            true,
        );
        struct TestCustomEntityParser {}

        impl CustomEntityParser for TestCustomEntityParser {
            fn extract_entities(
                &self,
                sentence: &str,
                filter_entity_kinds: Option<&[String]>,
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
    fn test_parse_slots_with_special_tokenized_out_characters() {
        // Given
        let text = "meeting with John O’reilly";
        let map = hashmap![
        hash_str_to_i32("meeting with % name %") => (0, vec![0]),
        ];
        let entity_scopes = vec![GroupedEntityScope {
            intent_group: vec!["intent1".to_string()],
            entity_scope: EntityScope {
                builtin: vec![],
                custom: vec!["name".to_string()],
            },
        }];
        let model = build_sample_model(
            vec!["name"],
            vec!["intent1"],
            map,
            entity_scopes,
            hashmap![],
            true,
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
        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
        let map = hashmap![
            hash_str_to_i32("hello % name %") => (0, vec![0]),
        ];
        let entity_scopes = vec![
            GroupedEntityScope {
                intent_group: vec!["greeting".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec!["name".to_string()],
                },
            },
            GroupedEntityScope {
                intent_group: vec!["other_intent".to_string()],
                entity_scope: EntityScope {
                    builtin: vec![],
                    custom: vec![],
                },
            },
        ];
        let model = build_sample_model(
            vec!["name"],
            vec!["greeting", "other_intent"],
            map,
            entity_scopes,
            hashmap![],
            false,
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

        let parser = LookupIntentParser::new(model, shared_resources).unwrap();

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
    fn test_replace_entities_with_placeholders() {
        // Given
        let text = "the third album of Blink 182 is great";
        let mut entities = vec![
            MatchedEntity {
                range: 0..9,
                entity_name: BuiltinEntityKind::Ordinal.identifier().to_string(),
            },
            MatchedEntity {
                range: 19..28,
                entity_name: BuiltinEntityKind::MusicArtist.identifier().to_string(),
            },
        ];

        // When
        let formatted_text = replace_entities_with_placeholders(text, &mut entities);

        // Then
        let expected_text = "%SNIPSORDINAL% album of %SNIPSMUSICARTIST% is great";
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
    fn test_get_combinations() {
        let expected_combinations = vec![
            vec![],
            vec![3, 0, 5],
            vec![3, 0],
            vec![3, 5],
            vec![0, 5],
            vec![3],
            vec![0],
            vec![5],
        ];
        assert_eq!(expected_combinations, get_items_combinations(vec![3, 0, 5]))
    }
}
