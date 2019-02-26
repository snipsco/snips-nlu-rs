use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::ResultExt;
use snips_nlu_ontology::{IntentClassifierResult, Language};
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_utils::string::substring_with_char_range;
use snips_nlu_utils::token::{tokenize, tokenize_light};

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::LookupParserModel;
use crate::resources::SharedResources;
use crate::slot_utils::*;
use crate::utils::{
    deduplicate_overlapping_entities, replace_entities, IntentName, MatchedEntity, SlotName,
};

use super::{IntentParser, InternalParsingResult};

/// HashMap based Intent Parser. The normalized/canonical form of an utterance
/// serves as the key and the value is tuple of (intent_id, [vec_of_slots_ids])
///
/// Once a lookup is done at inference, the intent and slots are retrieved by matching
/// their ids to a vecof intent names and a vec of slot names respectively.
pub struct LookupIntentParser {
    language: Language,
    slots_names: Vec<SlotName>,
    intents_names: Vec<IntentName>,
    map: HashMap<String, (i32, Vec<i32>)>,
    ignore_stop_words: bool,
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
        Ok(LookupIntentParser {
            language,
            slots_names: model.slots_names,
            intents_names: model.intents_names,
            map: model.map,
            ignore_stop_words: model.config.ignore_stop_words,
            shared_resources,
        })
    }
}

impl LookupIntentParser {
    fn generate_key(&self, input: &str) -> String {
        let lang = NluUtilsLanguage::from_language(self.language);
        let empty_set = HashSet::new();
        let stop_words = if self.ignore_stop_words {
            &self.shared_resources.stop_words
        } else {
            &empty_set
        };
        let key = tokenize_light(input, lang)
            .iter()
            .filter(|tkn| !stop_words.contains(tkn.as_str()))
            .fold(String::new(), |mut acc, tkn| {
                acc.push_str(tkn);
                acc
            });
        key.to_lowercase()
    }

    fn get_all_entities(&self, input: &str) -> Result<Vec<MatchedEntity>> {
        // get builtin entities
        let b = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(input, None, true)?
            .into_iter()
            .map(|entity| entity.into());
        // get custom entities
        let c = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(input, None)?
            .into_iter()
            .map(|entity| entity.into());

        // combine entities
        let mut all_entities: Vec<MatchedEntity> = Vec::new();
        all_entities.extend(b);
        all_entities.extend(c);

        Ok(deduplicate_overlapping_entities(all_entities))
    }

    fn empty_parsing_result(&self) -> InternalParsingResult {
        InternalParsingResult {
            intent: IntentClassifierResult {
                intent_name: None,
                probability: 1.0,
            },
            slots: vec![],
        }
    }

    fn parse_map_output(
        &self,
        input: &str,
        output: Option<&(i32, Vec<i32>)>,
        entities: Vec<MatchedEntity>,
        intents: Option<&[&str]>,
    ) -> InternalParsingResult {
        if let Some((intent_id, slots_ids)) = output {
            // assert invariant: length of slot ids matches that of entities
            debug_assert!(slots_ids.len() == entities.len());
            let intent_name = &self.intents_names[*intent_id as usize];
            if intents
                .unwrap_or(&[intent_name.as_ref()])
                .contains(&intent_name.as_ref())
            {
                let intent = IntentClassifierResult {
                    intent_name: Some(intent_name.to_string()),
                    probability: 1.0,
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
                InternalParsingResult { intent, slots }
            } else {
                // if intent name not in intents, return empty result
                self.empty_parsing_result()
            }
        } else {
            // if lookup value was none return empty result
            self.empty_parsing_result()
        }
    }
}

impl IntentParser for LookupIntentParser {
    fn parse(
        &self,
        input: &str,
        intents_whitelist: Option<&[&str]>,
    ) -> Result<InternalParsingResult> {
        let entities = self.get_all_entities(input)?;
        let (_, formatted_input) =
            replace_entities(input, entities.clone(), get_entity_placeholder);
        let cleaned_input = self.preprocess_text(input);
        let cleaned_formatted_input = self.preprocess_text(&*formatted_input);
        let key = self.generate_key(&cleaned_formatted_input);
        let val = if let Some(v) = self.map.get(&key) {
            Some(v)
        } else {
            self.map.get(&cleaned_input.to_lowercase())
        };

        Ok(self.parse_map_output(input, val, entities, intents_whitelist))
    }

    fn get_intents(&self, input: &str) -> Result<Vec<IntentClassifierResult>> {
        let mut intents = vec![];
        let res = self.parse(input, None)?;
        let names = if let Some(ref name) = res.intent.intent_name {
            intents.push(res.intent.clone());
            self.intents_names
                .iter()
                .filter(|x| *x != name)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            self.intents_names.clone()
        };
        for name in names {
            intents.push(IntentClassifierResult {
                intent_name: Some(name),
                probability: 0.0,
            })
        }
        intents.push(IntentClassifierResult {
            intent_name: None,
            probability: 0.0,
        });

        Ok(intents)
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
    fn preprocess_text(&self, string: &str) -> String {
        let tokens = tokenize(string, NluUtilsLanguage::from_language(self.language));
        let mut current_idx = 0;
        let mut cleaned_string = "".to_string();
        for mut token in tokens {
            if self.ignore_stop_words
                && self
                    .shared_resources
                    .stop_words
                    .contains(&token.normalized_value())
            {
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
}

impl fmt::Debug for LookupIntentParser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!(
            "{{\tlanguage: {:?}\n\t\
             slots_names: {:?}\n\t\
             intents_names: {:?}\n\t\
             map: {:#?}\n\tignore_stop_words: {:?}\n\t\
             shared_resources: Arc<..>\n}}",
            self.language, self.slots_names, self.intents_names, self.map, self.ignore_stop_words
        );
        write!(f, "{}", s)
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

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use std::collections::HashMap;
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

    fn test_configuration() -> LookupParserModel {
        LookupParserModel {
            language_code: "en".to_string(),
            slots_names: vec![
                "dummy_slot_name".to_string(),
                "dummy_slot_name2".to_string(),
                "dummy_slot_name3".to_string(),
                "dummy_slot_name4".to_string(),
            ],
            intents_names: vec![
                "dummy_intent_1".to_string(),
                "dummy_intent_2".to_string(),
                "dummy_intent_3".to_string(),
            ],
            map: hashmap![
                "%snipsdatetime%thereisa%dummy_entity_1%".to_string() => (0, vec![2, 0]),
                "thisisa%dummy_entity_1%".to_string() => (0, vec![0]),
                "thisisa%dummy_entity_1%querywithanother%dummy_entity_2%".to_string() => (0, vec![0, 1]),
                "thisisanother%dummy_entity_2%".to_string() => (0, vec![2]),
                "thisisanotherüber%dummy_entity_2%query!".to_string() => (0, vec![1]),
                "thisisa%dummy_entity_1%querywithanother%dummy_entity_2%%snipsdatetime%or%snipsdatetime%".to_string() => (0, vec![0,1,2,2]),
                "send%snipsamountofmoney%tojohn".to_string() => (2, vec![3]),
                "send%snipsamountofmoney%tojohnat%dummy_entity_2%".to_string() => (2, vec![3,1]),
            ],
            config: LookupParserConfig {
                ignore_stop_words: true,
            },
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
        let parsing_result = intent_parser
            .parse("make two cups of coffee", None)
            .unwrap();

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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_1".to_string()),
            probability: 1.0,
        };

        assert_eq!(expected_intent, intent);
    }

    #[test]
    fn test_parse_intent_with_whitelist() {
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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser
            .parse(text, Some(&["dummy_intent_2"]))
            .unwrap()
            .intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: None,
            probability: 1.0,
        };

        assert_eq!(intent, expected_intent);
    }

    #[test]
    fn test_get_intents() {
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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

        // When
        let intents = parser.get_intents(text).unwrap();

        // Then
        let first_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_1".to_string()),
            probability: 1.0,
        };
        assert_eq!(4, intents.len());
        assert_eq!(&first_intent, &intents[0]);
        assert_eq!(0.0, intents[1].probability);
        assert_eq!(0.0, intents[2].probability);
        assert_eq!(0.0, intents[3].probability);
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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_3".to_string()),
            probability: 1.0,
        };

        assert_eq!(intent, expected_intent);
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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

        // When
        let intent = parser.parse(text, None).unwrap().intent;

        // Then
        let expected_intent = IntentClassifierResult {
            intent_name: Some("dummy_intent_1".to_string()),
            probability: 1.0,
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
        let parser = LookupIntentParser::new(test_configuration(), shared_resources).unwrap();

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
        let parser = LookupIntentParser::new(test_configuration(), shared_resources).unwrap();

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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

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
        let parser = LookupIntentParser::new(test_configuration(), shared_resources).unwrap();

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
            LookupIntentParser::new(test_configuration(), Arc::new(shared_resources)).unwrap();

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

}
