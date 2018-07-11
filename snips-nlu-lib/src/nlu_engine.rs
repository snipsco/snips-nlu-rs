use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use itertools::Itertools;

use builtin_entity_parsing::{BuiltinEntityParserFactory, CachingBuiltinEntityParser};
use errors::*;
use failure::ResultExt;
use intent_parser::*;
use language::FromLanguage;
use models::{DatasetMetadata, Entity, NluEngineModel, ModelVersion, ProcessingUnitMetadata};
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::string::{normalize, substring_with_char_range};
use nlu_utils::token::{compute_all_ngrams, tokenize};
use resources::loading::load_resources;
use serde_json;
use slot_utils::resolve_builtin_slots;
use snips_nlu_ontology::{BuiltinEntityKind, IntentParserResult, Language, Slot, SlotValue};
use tempfile;
use zip::ZipArchive;

pub struct SnipsNluEngine {
    dataset_metadata: DatasetMetadata,
    parsers: Vec<Box<IntentParser>>,
    builtin_entity_parser: Arc<CachingBuiltinEntityParser>,
}

impl SnipsNluEngine {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let engine_model_path = path.as_ref().join("nlu_engine.json");
        Self::check_model_version(&engine_model_path)
            .with_context(|_|
                SnipsNluError::ModelLoad(engine_model_path.to_str().unwrap().to_string()))?;

        let resources_path = path.as_ref().join("resources");
        load_resources(resources_path)?;

        let model_file = fs::File::open(&engine_model_path)?;
        let model: NluEngineModel = serde_json::from_reader(model_file)?;

        let parsers = model
            .intent_parsers
            .iter()
            .map(|parser_name| {
                let parser_path = path.as_ref().join(parser_name);
                let metadata_path = parser_path.join("metadata.json");
                let metadata_file = fs::File::open(metadata_path)?;
                let metadata: ProcessingUnitMetadata = serde_json::from_reader(metadata_file)?;
                Ok(build_intent_parser(metadata, parser_path)? as _)
            })
            .collect::<Result<Vec<_>>>()?;

        let language = Language::from_str(&model.dataset_metadata.language_code)?;
        let builtin_entity_parser = BuiltinEntityParserFactory::get(language);

        Ok(SnipsNluEngine {
            dataset_metadata: model.dataset_metadata,
            parsers,
            builtin_entity_parser,
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
            .context("Could not load ZipBasedConfiguration")?;
        let temp_dir = tempfile::Builder::new()
            .prefix("temp_dir_nlu_")
            .tempdir()?;
        let temp_dir_path = temp_dir.path();

        let engine_dir_path = archive
            .by_index(0)?
            .sanitized_name();

        let engine_dir_name = engine_dir_path
            .to_str()
            .ok_or_else(|| format_err!("Engine directory name is empty"))?;

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

        Ok(SnipsNluEngine::from_path(temp_dir_path.join(engine_dir_name))?)
    }
}

impl SnipsNluEngine {
    pub fn parse(
        &self,
        input: &str,
        intents_filter: Option<&[String]>,
    ) -> Result<IntentParserResult> {
        if self.parsers.is_empty() {
            return Ok(IntentParserResult {
                input: input.to_string(),
                intent: None,
                slots: None,
            });
        }
        let set_intents: Option<HashSet<String>> = intents_filter
            .map(|intent_list| HashSet::from_iter(intent_list.iter().map(|name| name.to_string())));

        for parser in &self.parsers {
            let opt_internal_parsing_result = parser.parse(input, set_intents.as_ref())?;
            if let Some(internal_parsing_result) = opt_internal_parsing_result {
                let filter_entity_kinds = self.dataset_metadata
                    .slot_name_mappings
                    .values()
                    .flat_map::<Vec<_>, _>(|intent_mapping: &HashMap<String, String>| {
                        intent_mapping.values().collect()
                    })
                    .flat_map(|entity_name| BuiltinEntityKind::from_identifier(entity_name).ok())
                    .unique()
                    .collect::<Vec<_>>();

                let valid_slots = resolve_builtin_slots(
                    input,
                    internal_parsing_result.slots,
                    &*self.builtin_entity_parser,
                    Some(&*filter_entity_kinds),
                ).into_iter()
                    .filter_map(|slot| {
                        if let Some(entity) = self.dataset_metadata.entities.get(&slot.entity) {
                            entity
                                .utterances
                                .get(&normalize(&slot.raw_value))
                                .map(|reference_value| {
                                    Some(slot.clone().with_slot_value(SlotValue::Custom(
                                        reference_value.to_string().into(),
                                    )))
                                })
                                .unwrap_or(if entity.automatically_extensible {
                                    Some(slot)
                                } else {
                                    None
                                })
                        } else {
                            Some(slot)
                        }
                    })
                    .collect();

                return Ok(IntentParserResult {
                    input: input.to_string(),
                    intent: Some(internal_parsing_result.intent),
                    slots: Some(valid_slots),
                });
            }
        }
        Ok(IntentParserResult {
            input: input.to_string(),
            intent: None,
            slots: None,
        })
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
            let language = Language::from_str(&self.dataset_metadata.language_code)?;
            extract_custom_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                custom_entity,
                language,
            )
        } else {
            extract_builtin_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                &self.builtin_entity_parser,
            )?
        };
        Ok(slot)
    }
}

fn extract_custom_slot(
    input: String,
    entity_name: String,
    slot_name: String,
    custom_entity: &Entity,
    language: Language,
) -> Option<Slot> {
    let tokens = tokenize(&input, NluUtilsLanguage::from_language(language));
    let token_values_ref = tokens.iter().map(|v| &*v.value).collect_vec();
    let mut ngrams = compute_all_ngrams(&*token_values_ref, tokens.len());
    ngrams.sort_by_key(|&(_, ref indexes)| -(indexes.len() as i16));

    ngrams
        .into_iter()
        .find(|&(ref ngram, _)| custom_entity.utterances.contains_key(&normalize(ngram)))
        .map(|(ngram, _)| {
            Some(Slot {
                raw_value: ngram.clone(),
                value: SlotValue::Custom(
                    custom_entity.utterances[&normalize(&ngram)]
                        .to_string()
                        .into(),
                ),
                range: None,
                entity: entity_name.clone(),
                slot_name: slot_name.clone(),
            })
        })
        .unwrap_or(if custom_entity.automatically_extensible {
            Some(Slot {
                raw_value: input.clone(),
                value: SlotValue::Custom(input.into()),
                range: None,
                entity: entity_name,
                slot_name,
            })
        } else {
            None
        })
}

fn extract_builtin_slot(
    input: String,
    entity_name: String,
    slot_name: String,
    builtin_entity_parser: &CachingBuiltinEntityParser,
) -> Result<Option<Slot>> {
    let builtin_entity_kind = BuiltinEntityKind::from_identifier(&entity_name)?;
    Ok(builtin_entity_parser
        .extract_entities(&input, Some(&[builtin_entity_kind]), false)
        .first()
        .map(|rustlin_entity| Slot {
            raw_value: substring_with_char_range(input, &rustlin_entity.range),
            value: rustlin_entity.entity.clone(),
            range: None,
            entity: entity_name,
            slot_name,
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use snips_nlu_ontology::NumberValue;
    use utils::file_path;

    #[test]
    fn from_path_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("trained_engine");

        // When / Then
        let nlu_engine = SnipsNluEngine::from_path(path);
        assert!(nlu_engine.is_ok());
    }

    #[test]
    fn from_zip_works() {
        // Given
        let path = file_path("tests")
            .join("models")
            .join("trained_engine.zip");

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
            .join("trained_engine");
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
        let language = Language::EN;
        let input = "hello a b c d world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
            utterances: hashmap! {
                "a".to_string() => "value1".to_string(),
                "a b".to_string() => "value1".to_string(),
                "b c d".to_string() => "value2".to_string(),
            },
        };

        // When
        let extracted_slot =
            extract_custom_slot(input, entity_name, slot_name, &custom_entity, language);

        // Then
        let expected_slot = Some(Slot {
            raw_value: "b c d".to_string(),
            value: SlotValue::Custom("value2".to_string().into()),
            range: None,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_extract_custom_slot_when_not_tagged() {
        // Given
        let language = Language::EN;
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
            utterances: hashmap!{},
        };

        // When
        let extracted_slot =
            extract_custom_slot(input, entity_name, slot_name, &custom_entity, language);

        // Then
        let expected_slot = Some(Slot {
            raw_value: "hello world".to_string(),
            value: SlotValue::Custom("hello world".to_string().into()),
            range: None,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_not_extract_custom_slot_when_not_extensible() {
        // Given
        let language = Language::EN;
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: false,
            utterances: hashmap!{},
        };

        // When
        let extracted_slot =
            extract_custom_slot(input, entity_name, slot_name, &custom_entity, language);

        // Then
        let expected_slot = None;
        assert_eq!(expected_slot, extracted_slot);
    }
}
