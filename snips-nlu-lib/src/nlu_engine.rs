use std::collections::HashSet;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::{Component, Path};
use std::str::FromStr;
use std::sync::Arc;

use entity_parser::{BuiltinEntityParser, CustomEntityParser};
use errors::*;
use failure::ResultExt;
use intent_parser::*;
use itertools::Itertools;
use models::{DatasetMetadata, Entity, ModelVersion, NluEngineModel, ProcessingUnitMetadata};
use nlu_utils::string::substring_with_char_range;
use resources::loading::load_shared_resources;
use resources::SharedResources;
use slot_utils::*;
use serde_json;
use snips_nlu_ontology::{BuiltinEntityKind, IntentParserResult, Language, Slot, SlotValue};
use tempfile;
use utils::{EntityName, IntentName, SlotName};
use zip::ZipArchive;

pub struct SnipsNluEngine {
    dataset_metadata: DatasetMetadata,
    intent_parsers: Vec<Box<IntentParser>>,
    shared_resources: Arc<SharedResources>,
}

impl SnipsNluEngine {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let engine_model_path = path.as_ref().join("nlu_engine.json");
        Self::check_model_version(&engine_model_path)
            .with_context(|_|
                SnipsNluError::ModelLoad(engine_model_path.to_str().unwrap().to_string()))?;


        let model_file = fs::File::open(&engine_model_path)
            .with_context(|_| format!("Could not open nlu engine file {:?}", &engine_model_path))?;
        let model: NluEngineModel = serde_json::from_reader(model_file)
            .with_context(|_| "Could not deserialize nlu engine json file")?;

        let language = Language::from_str(&model.dataset_metadata.language_code)?;
        let resources_path = path.as_ref().join("resources").join(language.to_string());
        let builtin_parser_path = path.as_ref().join(&model.builtin_entity_parser);
        let custom_parser_path = path.as_ref().join(&model.custom_entity_parser);
        let shared_resources = load_shared_resources(&resources_path, builtin_parser_path, custom_parser_path)?;

        let parsers = model
            .intent_parsers
            .iter()
            .map(|parser_name| {
                let parser_path = path.as_ref().join(parser_name);
                let metadata_path = parser_path.join("metadata.json");
                let metadata_file = fs::File::open(metadata_path)
                    .with_context(|_|
                        format!("Could not open metadata file of parser '{}'", parser_name))?;
                let metadata: ProcessingUnitMetadata = serde_json::from_reader(metadata_file)
                    .with_context(|_|
                        format!("Could not deserialize json metadata of parser '{}'", parser_name))?;
                Ok(build_intent_parser(metadata, parser_path, shared_resources.clone())? as _)
            })
            .collect::<Result<Vec<_>>>()?;


        Ok(SnipsNluEngine {
            dataset_metadata: model.dataset_metadata,
            intent_parsers: parsers,
            shared_resources,
        })
    }

    fn check_model_version<P: AsRef<Path>>(path: P) -> Result<()> {
        let model_file = fs::File::open(&path)?;

        let model_version: ModelVersion = ::serde_json::from_reader(model_file)?;
        if model_version.model_version != ::MODEL_VERSION {
            bail!(SnipsNluError::WrongModelVersion(
                model_version.model_version,
                ::MODEL_VERSION
            ));
        }
        Ok(())
    }
}

impl SnipsNluEngine {
    pub fn from_zip<R: io::Read + io::Seek>(reader: R) -> Result<Self> {
        let mut archive = ZipArchive::new(reader)
            .with_context(|_| "Could not read nlu engine zip data")?;
        let temp_dir = tempfile::Builder::new()
            .prefix("temp_dir_nlu_")
            .tempdir()?;
        let temp_dir_path = temp_dir.path();

        for file_index in 0..archive.len() {
            let mut file = archive.by_index(file_index)?;
            let outpath = temp_dir_path.join(file.sanitized_name());

            if (&*file.name()).ends_with('/') || (&*file.name()).ends_with('\\') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile)?;
            }
        }

        let first_archive_file = archive
            .by_index(0)?
            .sanitized_name();

        let engine_dir_path = first_archive_file
            .components()
            .find(|component| if let Component::Normal(_) = component { true } else { false })
            .ok_or_else(|| format_err!("Trained engine archive is incorrect"))?
            .as_os_str();

        let engine_dir_name = engine_dir_path
            .to_str()
            .ok_or_else(|| format_err!("Engine directory name is empty"))?;

        Ok(SnipsNluEngine::from_path(temp_dir_path.join(engine_dir_name))?)
    }
}

impl SnipsNluEngine {
    pub fn parse(
        &self,
        input: &str,
        intents_filter: Option<&[IntentName]>,
    ) -> Result<IntentParserResult> {
        if self.intent_parsers.is_empty() {
            return Ok(IntentParserResult {
                input: input.to_string(),
                intent: None,
                slots: None,
            });
        }
        let set_intents: Option<HashSet<IntentName>> = intents_filter
            .map(|intent_list| HashSet::from_iter(intent_list.iter().map(|name| name.to_string())));

        for parser in &self.intent_parsers {
            let opt_internal_parsing_result = parser.parse(input, set_intents.as_ref())?;
            if let Some(internal_parsing_result) = opt_internal_parsing_result {
                let intent = internal_parsing_result.intent;
                let builtin_entity_scope = self.dataset_metadata
                    .slot_name_mappings
                    .get(&intent.intent_name)
                    .ok_or_else(|| format_err!("Cannot find intent '{}' in dataset metadata", &intent.intent_name))?
                    .values()
                    .flat_map(|entity_name| BuiltinEntityKind::from_identifier(entity_name).ok())
                    .unique()
                    .collect::<Vec<_>>();

                let custom_entity_scope: Vec<String> = self.dataset_metadata
                    .entities
                    .keys()
                    .map(|entity| entity.to_string())
                    .collect();

                let resolved_slots = self.resolve_slots(
                    input,
                    internal_parsing_result.slots,
                    Some(&*builtin_entity_scope),
                    Some(&*custom_entity_scope),
                ).with_context(|_| format!("Cannot resolve slots"))?;

                return Ok(IntentParserResult {
                    input: input.to_string(),
                    intent: Some(intent),
                    slots: Some(resolved_slots),
                });
            }
        }
        Ok(IntentParserResult {
            input: input.to_string(),
            intent: None,
            slots: None,
        })
    }

    fn resolve_slots(
        &self,
        text: &str,
        slots: Vec<InternalSlot>,
        builtin_entity_filter: Option<&[BuiltinEntityKind]>,
        custom_entity_filter: Option<&[EntityName]>,
    ) -> Result<Vec<Slot>> {
        let builtin_entities = self.shared_resources.builtin_entity_parser
            .extract_entities(text, builtin_entity_filter, false)?;
        let custom_entities = self.shared_resources.custom_entity_parser
            .extract_entities(text, custom_entity_filter, true)?;

        let mut resolved_slots = Vec::with_capacity(slots.len());
        for slot in slots.into_iter() {
            let opt_resolved_slot = if let Some(entity) = self.dataset_metadata.entities.get(&slot.entity) {
                resolve_custom_slot(
                    slot,
                    &entity,
                    &custom_entities,
                    self.shared_resources.custom_entity_parser.clone())?
            } else {
                resolve_builtin_slot(
                    slot,
                    &builtin_entities,
                    self.shared_resources.builtin_entity_parser.clone())?
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
        let entity_name = self.dataset_metadata
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
                self.shared_resources.custom_entity_parser.clone())?
        } else {
            extract_builtin_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                self.shared_resources.builtin_entity_parser.clone())?
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
    let mut custom_entities = custom_entity_parser.extract_entities(&input, Some(&[entity_name.clone()]), true)?;
    Ok(if let Some(matched_entity) = custom_entities.pop() {
        Some(Slot {
            raw_value: matched_entity.value,
            value: SlotValue::Custom(matched_entity.resolved_value.into()),
            range: Some(matched_entity.range),
            entity: entity_name.clone(),
            slot_name: slot_name.clone(),
        })
    } else if custom_entity.automatically_extensible {
        let range = Some(0..input.chars().count());
        Some(Slot {
            raw_value: input.clone(),
            value: SlotValue::Custom(input.into()),
            range,
            entity: entity_name,
            slot_name,
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
        .first()
        .map(|builtin_entity| Slot {
            raw_value: substring_with_char_range(input, &builtin_entity.range),
            value: builtin_entity.entity.clone(),
            range: None,
            entity: entity_name,
            slot_name,
        }))
}

#[cfg(test)]
mod tests {
    use snips_nlu_ontology::NumberValue;
    use super::*;
    use entity_parser::custom_entity_parser::CustomEntity;
    use testutils::*;

    #[test]
    fn from_path_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("nlu_engine");

        // When / Then
        let nlu_engine = SnipsNluEngine::from_path(path);
        assert!(nlu_engine.is_ok());
    }

    #[test]
    fn from_zip_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("nlu_engine.zip");

        let file = fs::File::open(path).unwrap();

        // When
        let nlu_engine = SnipsNluEngine::from_zip(file);

        // Then
        assert!(nlu_engine.is_ok());

        let result = nlu_engine
            .unwrap()
            .parse("Make me two cups of coffee please", None)
            .unwrap();

        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = Some(vec![
            Slot {
                raw_value: "two".to_string(),
                value: expected_entity_value,
                range: Some(8..11),
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            },
        ]);
        let expected_intent = Some("MakeCoffee".to_string());

        assert_eq!(expected_intent, result.intent.map(|intent| intent.intent_name));
        assert_eq!(expected_slots, result.slots);
    }

    #[test]
    fn parse_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("nlu_engine");
        let nlu_engine = SnipsNluEngine::from_path(path).unwrap();

        // When
        let result = nlu_engine
            .parse("Make me two cups of coffee please", None)
            .unwrap();

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_slots = Some(vec![
            Slot {
                raw_value: "two".to_string(),
                value: expected_entity_value,
                range: Some(8..11),
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            },
        ]);
        let expected_intent = Some("MakeCoffee".to_string());

        assert_eq!(expected_intent, result.intent.map(|intent| intent.intent_name));
        assert_eq!(expected_slots, result.slots);
    }

    #[test]
    fn should_extract_custom_slot_when_tagged() {
        // Given
        let input = "hello a b c d world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
        };

        let mocked_custom_parser = Arc::new(MockedCustomEntityParser::from_iter(
            vec![(
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
                    }
                ]
            )]
        ));

        // When
        let extracted_slot = extract_custom_slot(
            input, entity_name, slot_name, &custom_entity, mocked_custom_parser).unwrap();

        // Then
        let expected_slot = Some(Slot {
            raw_value: "b c d".to_string(),
            value: SlotValue::Custom("value2".to_string().into()),
            range: Some(8..13),
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_extract_custom_slot_when_not_tagged() {
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
            input, entity_name, slot_name, &custom_entity, mocked_custom_parser).unwrap();

        // Then
        let expected_slot = Some(Slot {
            raw_value: "hello world".to_string(),
            value: SlotValue::Custom("hello world".to_string().into()),
            range: Some(0..11),
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_not_extract_custom_slot_when_not_extensible() {
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
            input, entity_name, slot_name, &custom_entity, mocked_custom_parser).unwrap();

        // Then
        let expected_slot = None;
        assert_eq!(expected_slot, extracted_slot);
    }
}
