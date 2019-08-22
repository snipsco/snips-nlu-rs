use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::{bail, format_err, ResultExt};
use itertools::Itertools;
use snips_nlu_ontology::{
    BuiltinEntityKind, IntentClassifierResult, IntentParserResult, Language, Slot, SlotValue,
};
use snips_nlu_utils::string::substring_with_char_range;

use crate::entity_parser::{BuiltinEntityParser, CustomEntityParser};
use crate::errors::*;
use crate::intent_parser::*;
use crate::models::{
    DatasetMetadata, Entity, ModelVersion, NluEngineModel, ProcessingUnitMetadata,
};
use crate::ontology::IntentParserAlternative;
use crate::resources::loading::load_shared_resources;
use crate::resources::SharedResources;
use crate::slot_utils::*;
use crate::utils::{extract_nlu_engine_zip_archive, EntityName, IterOps, SlotName};

pub struct SnipsNluEngine {
    dataset_metadata: DatasetMetadata,
    intent_parsers: Vec<Box<IntentParser>>,
    shared_resources: Arc<SharedResources>,
}

impl SnipsNluEngine {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let model = SnipsNluEngine::load_model(&path)?;

        let language = Language::from_str(&model.dataset_metadata.language_code)?;

        let resources_path = path.as_ref().join("resources").join(language.to_string());
        let builtin_parser_path = path.as_ref().join(&model.builtin_entity_parser);
        let custom_parser_path = path.as_ref().join(&model.custom_entity_parser);

        let shared_resources =
            load_shared_resources(&resources_path, builtin_parser_path, custom_parser_path)?;

        let parsers = Self::load_intent_parsers(path, &model, shared_resources.clone())?;

        Ok(SnipsNluEngine {
            dataset_metadata: model.dataset_metadata,
            intent_parsers: parsers,
            shared_resources,
        })
    }

    fn check_model_version<P: AsRef<Path>>(path: P) -> Result<()> {
        let model_file = fs::File::open(&path)?;

        let model_version: ModelVersion = serde_json::from_reader(model_file)?;
        if model_version.model_version != crate::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion {
                model: model_version.model_version,
                runner: crate::MODEL_VERSION
            });
        }
        Ok(())
    }

    fn load_model<P: AsRef<Path>>(path: P) -> Result<NluEngineModel> {
        let engine_model_path = path.as_ref().join("nlu_engine.json");
        Self::check_model_version(&engine_model_path).with_context(|_| {
            SnipsNluError::ModelLoad(engine_model_path.to_str().unwrap().to_string())
        })?;
        let model_file = fs::File::open(&engine_model_path)
            .with_context(|_| format!("Could not open nlu engine file {:?}", &engine_model_path))?;
        let model = serde_json::from_reader(model_file)
            .with_context(|_| format!("Invalid nlu engine file {:?}", &engine_model_path))?;
        Ok(model)
    }

    fn load_intent_parsers<P: AsRef<Path>>(
        engine_dir: P,
        model: &NluEngineModel,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<IntentParser>>> {
        model
            .intent_parsers
            .iter()
            .map(|parser_name| {
                let parser_path = engine_dir.as_ref().join(parser_name);
                let metadata_path = parser_path.join("metadata.json");
                let metadata_file = fs::File::open(metadata_path).with_context(|_| {
                    format!("Could not open metadata file of parser '{}'", parser_name)
                })?;
                let metadata: ProcessingUnitMetadata = serde_json::from_reader(metadata_file)
                    .with_context(|_| {
                        format!(
                            "Could not deserialize json metadata of parser '{}'",
                            parser_name
                        )
                    })?;
                Ok(build_intent_parser(metadata, parser_path, shared_resources.clone())? as _)
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[cfg(test)]
impl SnipsNluEngine {
    pub fn from_path_with_resources<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let model = SnipsNluEngine::load_model(&path)?;
        let parsers = Self::load_intent_parsers(path, &model, shared_resources.clone())?;

        Ok(SnipsNluEngine {
            dataset_metadata: model.dataset_metadata,
            intent_parsers: parsers,
            shared_resources,
        })
    }
}

impl SnipsNluEngine {
    pub fn from_zip<R: io::Read + io::Seek>(reader: R) -> Result<Self> {
        let temp_dir = tempfile::Builder::new().prefix("temp_dir_nlu_").tempdir()?;
        let temp_dir_path = temp_dir.path();
        let engine_dir_path = extract_nlu_engine_zip_archive(reader, temp_dir_path)?;
        Ok(SnipsNluEngine::from_path(engine_dir_path)?)
    }
}

impl SnipsNluEngine {
    pub fn parse<'a, 'b, W, B>(
        &self,
        input: &str,
        intents_whitelist: W,
        intents_blacklist: B,
    ) -> Result<IntentParserResult>
    where
        W: Into<Option<Vec<&'a str>>>,
        B: Into<Option<Vec<&'b str>>>,
    {
        let intents_whitelist_owned =
            self.get_intents_whitelist(intents_whitelist, intents_blacklist)?;
        let intents_whitelist = intents_whitelist_owned
            .as_ref()
            .map(|whitelist| whitelist.as_ref());
        let mut none_score: f32 = 0.0;
        for parser in &self.intent_parsers {
            let internal_parsing_result = parser.parse(input, intents_whitelist)?;
            if internal_parsing_result.intent.intent_name.is_some() {
                let resolved_slots = self
                    .resolve_slots(input, internal_parsing_result.slots)
                    .with_context(|_| "Cannot resolve slots".to_string())?;

                return Ok(IntentParserResult {
                    input: input.to_string(),
                    intent: internal_parsing_result.intent,
                    slots: resolved_slots,
                    alternatives: vec![],
                });
            } else {
                none_score = internal_parsing_result.intent.confidence_score;
            }
        }

        // If all parsers failed to extract an intent, we use the confidence score returned by the last
        // parser
        Ok(IntentParserResult {
            input: input.to_string(),
            intent: IntentClassifierResult {
                intent_name: None,
                confidence_score: none_score,
            },
            slots: vec![],
            alternatives: vec![],
        })
    }

    pub fn parse_with_alternatives<'a, 'b, W, B>(
        &self,
        input: &str,
        intents_whitelist: W,
        intents_blacklist: B,
        intents_alternatives: usize,
    ) -> Result<IntentParserResult>
    where
        W: Into<Option<Vec<&'a str>>>,
        B: Into<Option<Vec<&'b str>>>,
    {
        let intents_whitelist_owned =
            self.get_intents_whitelist(intents_whitelist, intents_blacklist)?;
        let mut intent_results = self.parse(input, intents_whitelist_owned.clone(), None)?;
        if intents_alternatives == 0 {
            return Ok(intent_results);
        }
        let intents_whitelist = intents_whitelist_owned
            .as_ref()
            .map(|whitelist| whitelist.as_ref());

        let alternative_results: Vec<IntentParserAlternative> = self
            .get_intents(input)?
            .into_iter()
            .filter(|res| {
                res.intent_name
                    .as_ref()
                    .map(|name| {
                        intents_whitelist
                            .map(|whitelist: &[&str]| whitelist.contains(&&**name))
                            .unwrap_or(true)
                    })
                    .unwrap_or(true)
            })
            .skip(1) // We do not duplicate the top result in the list of alternatives
            .take(intents_alternatives)
            .map(|res| {
                res.intent_name
                    .as_ref()
                    .map(|intent_name| Ok(self.get_slots(input, intent_name)?))
                    .unwrap_or_else(|| Ok(vec![]))
                    .map(|slots| IntentParserAlternative { intent: res, slots })
            })
            .collect::<Result<Vec<_>>>()?;

        intent_results.alternatives = alternative_results;
        Ok(intent_results)
    }

    fn get_intents_whitelist<'a: 'c, 'b: 'c, 'c, W, B>(
        &'c self,
        intents_whitelist: W,
        intents_blacklist: B,
    ) -> Result<Option<Vec<&'c str>>>
    where
        W: Into<Option<Vec<&'a str>>>,
        B: Into<Option<Vec<&'b str>>>,
    {
        let all_intents: HashSet<&str> = self
            .dataset_metadata
            .slot_name_mappings
            .keys()
            .map(|intent| &**intent)
            .collect();
        let intents_whitelist = intents_whitelist.into();
        let intents_blacklist = intents_blacklist.into();
        if let Some(unknown_intent) = vec![intents_whitelist.as_ref(), intents_blacklist.as_ref()]
            .into_iter()
            .flatten()
            .flatten()
            .find(|intent| !all_intents.contains(*intent))
        {
            return Err(format_err!(
                "Cannot use unknown intent '{}' in intents filter",
                unknown_intent
            ));
        };
        let reverted_whitelist: Option<Vec<&str>> = intents_blacklist.map(|blacklist| {
            all_intents
                .iter()
                .filter(|intent_name| !blacklist.contains(intent_name))
                .cloned()
                .collect()
        });
        Ok(intents_whitelist
            .map(|whitelist| {
                reverted_whitelist
                    .clone()
                    .map(|rev_whitelist| rev_whitelist.intersect(whitelist.clone()))
                    .unwrap_or_else(|| whitelist)
            })
            .or_else(|| reverted_whitelist))
    }

    pub fn get_intents(&self, input: &str) -> Result<Vec<IntentClassifierResult>> {
        let nb_intents = self.dataset_metadata.slot_name_mappings.len();
        let mut results = HashMap::with_capacity(nb_intents + 1);
        for parser in self.intent_parsers.iter() {
            let parser_results = parser.get_intents(input)?;
            if results.is_empty() {
                for res in parser_results.into_iter() {
                    results.insert(res.intent_name.clone(), res);
                }
                continue;
            }
            for res in parser_results.into_iter() {
                let existing_score = results
                    .get(&res.intent_name)
                    .map(|r| r.confidence_score)
                    .unwrap_or(0.0);
                if res.confidence_score > existing_score {
                    results
                        .entry(res.intent_name.clone())
                        .and_modify(|e| e.confidence_score = res.confidence_score)
                        .or_insert_with(|| res);
                }
            }
        }
        Ok(results
            .into_iter()
            .map(|(_, res)| res)
            .sorted_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap())
            .collect())
    }

    pub fn get_slots(&self, input: &str, intent: &str) -> Result<Vec<Slot>> {
        for parser in &self.intent_parsers {
            let slots = parser.get_slots(input, intent)?;
            if !slots.is_empty() {
                return self.resolve_slots(input, slots);
            }
        }
        Ok(vec![])
    }

    fn resolve_slots(&self, text: &str, slots: Vec<InternalSlot>) -> Result<Vec<Slot>> {
        let builtin_entity_scope: Vec<BuiltinEntityKind> = slots
            .iter()
            .filter_map(|slot| BuiltinEntityKind::from_str(&slot.entity).ok())
            .collect();
        let custom_entity_scope: Vec<String> = slots
            .iter()
            .filter_map(|slot| {
                if BuiltinEntityKind::from_str(&slot.entity).is_ok() {
                    None
                } else {
                    Some(slot.entity.to_string())
                }
            })
            .collect();
        let builtin_entities = self
            .shared_resources
            .builtin_entity_parser
            .extract_entities(text, Some(&*builtin_entity_scope), false)?;
        let custom_entities = self
            .shared_resources
            .custom_entity_parser
            .extract_entities(text, Some(&*custom_entity_scope))?;

        let mut resolved_slots = Vec::with_capacity(slots.len());
        for slot in slots.into_iter() {
            let opt_resolved_slot =
                if let Some(entity) = self.dataset_metadata.entities.get(&slot.entity) {
                    resolve_custom_slot(
                        slot,
                        &entity,
                        &custom_entities,
                        self.shared_resources.custom_entity_parser.clone(),
                    )?
                } else {
                    resolve_builtin_slot(
                        slot,
                        &builtin_entities,
                        self.shared_resources.builtin_entity_parser.clone(),
                    )?
                };
            if let Some(resolved_slot) = opt_resolved_slot {
                resolved_slots.push(resolved_slot);
            }
        }
        Ok(resolved_slots)
    }
}

impl SnipsNluEngine {
    pub fn extract_slot(
        &self,
        input: String,
        intent_name: &str,
        slot_name: &str,
    ) -> Result<Option<Slot>> {
        let entity_name = self
            .dataset_metadata
            .slot_name_mappings
            .get(intent_name)
            .ok_or_else(|| format_err!("Unknown intent: {}", intent_name))?
            .get(slot_name)
            .ok_or_else(|| format_err!("Unknown slot: {}", &slot_name))?;

        let slot = if let Some(custom_entity) = self.dataset_metadata.entities.get(entity_name) {
            extract_custom_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                custom_entity,
                self.shared_resources.custom_entity_parser.clone(),
            )?
        } else {
            extract_builtin_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                self.shared_resources.builtin_entity_parser.clone(),
            )?
        };
        Ok(slot)
    }
}

fn extract_custom_slot(
    input: String,
    entity_name: EntityName,
    slot_name: SlotName,
    custom_entity: &Entity,
    custom_entity_parser: Arc<CustomEntityParser>,
) -> Result<Option<Slot>> {
    let mut custom_entities =
        custom_entity_parser.extract_entities(&input, Some(&[entity_name.clone()]))?;
    Ok(if let Some(matched_entity) = custom_entities.pop() {
        Some(Slot {
            raw_value: matched_entity.value,
            value: SlotValue::Custom(matched_entity.resolved_value.into()),
            range: matched_entity.range,
            entity: entity_name.clone(),
            slot_name: slot_name.clone(),
            confidence_score: None,
        })
    } else if custom_entity.automatically_extensible {
        let range = 0..input.chars().count();
        Some(Slot {
            raw_value: input.clone(),
            value: SlotValue::Custom(input.into()),
            range,
            entity: entity_name,
            slot_name,
            confidence_score: None,
        })
    } else {
        None
    })
}

fn extract_builtin_slot(
    input: String,
    entity_name: EntityName,
    slot_name: SlotName,
    builtin_entity_parser: Arc<BuiltinEntityParser>,
) -> Result<Option<Slot>> {
    let builtin_entity_kind = BuiltinEntityKind::from_identifier(&entity_name)?;
    Ok(builtin_entity_parser
        .extract_entities(&input, Some(&[builtin_entity_kind]), false)?
        .into_iter()
        .next()
        .map(|builtin_entity| Slot {
            raw_value: substring_with_char_range(input, &builtin_entity.range),
            value: builtin_entity.entity,
            range: builtin_entity.range,
            entity: entity_name,
            slot_name,
            confidence_score: None,
        }))
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use snips_nlu_ontology::{NumberValue, StringValue};

    use crate::entity_parser::custom_entity_parser::CustomEntity;
    use crate::testutils::*;

    use super::*;

    #[test]
    fn test_load_from_zip() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine.zip");

        let file = fs::File::open(path).unwrap();

        // When
        let nlu_engine = SnipsNluEngine::from_zip(file);

        // Then
        assert!(nlu_engine.is_ok());

        let result = nlu_engine
            .unwrap()
            .parse("Make me two cups of coffee please", None, None)
            .unwrap();

        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = vec![Slot {
            raw_value: "two".to_string(),
            value: expected_entity_value,
            range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
            confidence_score: None,
        }];
        let expected_intent = Some("MakeCoffee".to_string());

        assert_eq!(expected_intent, result.intent.intent_name);
        assert_eq!(expected_slots, result.slots);
    }

    #[test]
    fn test_parse() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let result = nlu_engine
            .parse("Make me two cups of coffee please", None, None)
            .unwrap();

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = vec![Slot {
            raw_value: "two".to_string(),
            value: expected_entity_value,
            range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
            confidence_score: None,
        }];
        let expected_intent = Some("MakeCoffee".to_string());

        assert_eq!(expected_intent, result.intent.intent_name);
        assert_eq!(expected_slots, result.slots);
    }

    #[test]
    fn test_parse_with_whitelist_and_blacklist() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let whitelist_result = nlu_engine
            .parse("Make me two cups of coffee please", vec!["MakeTea"], None)
            .unwrap();

        let blacklist_result = nlu_engine
            .parse(
                "Make me two cups of coffee please",
                None,
                vec!["MakeCoffee"],
            )
            .unwrap();

        let combined_result = nlu_engine
            .parse(
                "Make me two cups of coffee please",
                vec!["MakeCoffee", "MakeTea"],
                vec!["MakeCoffee"],
            )
            .unwrap();

        let failed_result = nlu_engine.parse(
            "Make me two cups of coffee please",
            vec!["MakeCoffee"],
            vec!["MakeChocolate"],
        );
        let failed_result_2 = nlu_engine.parse(
            "Make me two cups of coffee please",
            vec!["MakeChocolate"],
            vec!["MakeCoffee"],
        );

        // Then
        assert_eq!(
            Some("MakeTea".to_string()),
            whitelist_result.intent.intent_name
        );
        assert_eq!(
            Some("MakeTea".to_string()),
            blacklist_result.intent.intent_name
        );
        assert_eq!(
            Some("MakeTea".to_string()),
            combined_result.intent.intent_name
        );
        assert!(failed_result.is_err());
        assert!(failed_result_2.is_err());
    }

    #[test]
    fn test_parse_with_alternatives() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let mut result = nlu_engine
            .parse_with_alternatives("Make me two cups of coffee please", None, None, 1)
            .unwrap();

        // set the confidence scores to 0.5 for testability
        result.intent.confidence_score = 0.8;
        for alternative in result.alternatives.iter_mut() {
            alternative.intent.confidence_score = 0.5;
        }

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = vec![Slot {
            raw_value: "two".to_string(),
            value: expected_entity_value,
            range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
            confidence_score: None,
        }];
        let expected_alternatives: Vec<IntentParserAlternative> = vec![IntentParserAlternative {
            intent: IntentClassifierResult {
                intent_name: Some("MakeTea".to_string()),
                confidence_score: 0.5,
            },
            slots: vec![Slot {
                raw_value: "two".to_string(),
                value: SlotValue::Number(NumberValue { value: 2.0 }),
                range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
                confidence_score: None,
            }],
        }];

        let expected_result = IntentParserResult {
            input: "Make me two cups of coffee please".to_string(),
            intent: IntentClassifierResult {
                intent_name: Some("MakeCoffee".to_string()),
                confidence_score: 0.8,
            },
            slots: expected_slots,
            alternatives: expected_alternatives,
        };
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_parse_with_whitelist_and_alternatives() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let mut result = nlu_engine
            .parse_with_alternatives(
                "Make me two cups of coffee please",
                vec!["MakeCoffee"],
                None,
                1,
            )
            .unwrap();

        // set the confidence scores to 0.5 for testability
        result.intent.confidence_score = 0.8;
        for alternative in result.alternatives.iter_mut() {
            alternative.intent.confidence_score = 0.5;
        }

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = vec![Slot {
            raw_value: "two".to_string(),
            value: expected_entity_value,
            range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
            confidence_score: None,
        }];
        let expected_alternatives: Vec<IntentParserAlternative> = vec![IntentParserAlternative {
            intent: IntentClassifierResult {
                intent_name: None,
                confidence_score: 0.5,
            },
            slots: vec![],
        }];

        let expected_result = IntentParserResult {
            input: "Make me two cups of coffee please".to_string(),
            intent: IntentClassifierResult {
                intent_name: Some("MakeCoffee".to_string()),
                confidence_score: 0.8,
            },
            slots: expected_slots,
            alternatives: expected_alternatives,
        };
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_get_intents() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let intents: Vec<Option<String>> = nlu_engine
            .get_intents("Make me two hot cups of tea")
            .unwrap()
            .into_iter()
            .map(|intent| intent.intent_name)
            .collect();

        // Then
        let expected_intents = vec![
            Some("MakeTea".to_string()),
            None,
            Some("MakeCoffee".to_string()),
        ];
        assert_eq!(expected_intents, intents);
    }

    #[test]
    fn test_get_slots() {
        // Given
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let slots = nlu_engine
            .get_slots("Make me two hot cups of tea", "MakeTea")
            .unwrap();

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = vec![
            Slot {
                raw_value: "two".to_string(),
                value: expected_entity_value,
                range: 8..11,
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
                confidence_score: None,
            },
            Slot {
                raw_value: "hot".to_string(),
                value: SlotValue::Custom(StringValue {
                    value: "hot".to_string(),
                }),
                range: 12..15,
                entity: "Temperature".to_string(),
                slot_name: "beverage_temperature".to_string(),
                confidence_score: None,
            },
        ];
        assert_eq!(expected_slots, slots);
    }

    #[test]
    fn test_extract_custom_slot_when_tagged() {
        // Given
        let input = "hello a b c d world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
        };

        let mocked_custom_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![(
            "hello a b c d world".to_string(),
            vec![
                CustomEntity {
                    value: "a b".to_string(),
                    resolved_value: "value1".to_string(),
                    range: 6..9,
                    entity_identifier: entity_name.to_string(),
                },
                CustomEntity {
                    value: "b c d".to_string(),
                    resolved_value: "value2".to_string(),
                    range: 8..13,
                    entity_identifier: entity_name.to_string(),
                },
            ],
        )]));

        // When
        let extracted_slot = extract_custom_slot(
            input,
            entity_name,
            slot_name,
            &custom_entity,
            mocked_custom_parser,
        )
        .unwrap();

        // Then
        let expected_slot = Some(Slot {
            raw_value: "b c d".to_string(),
            value: SlotValue::Custom("value2".to_string().into()),
            range: 8..13,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn test_extract_custom_slot_when_not_tagged() {
        // Given
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
        };

        let mocked_custom_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let extracted_slot = extract_custom_slot(
            input,
            entity_name,
            slot_name,
            &custom_entity,
            mocked_custom_parser,
        )
        .unwrap();

        // Then
        let expected_slot = Some(Slot {
            raw_value: "hello world".to_string(),
            value: SlotValue::Custom("hello world".to_string().into()),
            range: 0..11,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn test_do_not_extract_custom_slot_when_not_extensible() {
        // Given
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: false,
        };

        let mocked_custom_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let extracted_slot = extract_custom_slot(
            input,
            entity_name,
            slot_name,
            &custom_entity,
            mocked_custom_parser,
        )
        .unwrap();

        // Then
        let expected_slot = None;
        assert_eq!(expected_slot, extracted_slot);
    }
}
