use std::collections::{HashMap, HashSet};
use std::fs;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

use entity_parser::custom_entity_parser::CustomEntityParserMetadata;
use failure::ResultExt;
use injection::errors::{NluInjectionError, NluInjectionErrorKind};
use models::nlu_engine::NluEngineModel;
use snips_nlu_ontology::{BuiltinGazetteerEntityKind, GrammarEntityKind};
use snips_nlu_ontology_parsers::{BuiltinParserMetadata, GazetteerParserMetadata};
use snips_nlu_ontology_parsers::gazetteer_entity_parser::{Parser as GazetterEntityParser,
                                                          EntityValue as GazetteerEntityValue};

pub type InjectedEntity = String;
pub type InjectedValue = String;

fn normalize(s: &str) -> String {
    s.to_lowercase()
}

struct NLUEngineInfo {
    builtin_entity_parser_dir: PathBuf,
    custom_entity_parser_dir: PathBuf,
    custom_entities: HashSet<InjectedEntity>,
}

struct GazetteerParserInfo {
    gazetteer_parser_dir: PathBuf,
    gazetteer_parser_metadata: GazetteerParserMetadata,
}

// TODO: HANDLE STEMMING FOR CUSTOM ENTITIES
pub fn inject_entity_values<P: AsRef<Path>>(
    nlu_engine_dir: P,
    entity_values: &HashMap<InjectedEntity, Vec<InjectedValue>>,
    from_vanilla: bool,
) -> Result<(), NluInjectionError> {
    info!("Starting injection...");

    info!("Retrieving parsers paths...");
    let parsers_dirs = get_entity_parsers_dirs(nlu_engine_dir.as_ref(), entity_values)?;

    info!("Normalizing injected values...");
    // Update all values
    let normalized_entity_values: HashMap<InjectedEntity, Vec<GazetteerEntityValue>> = entity_values
        .into_iter()
        .map(|(entity, values)| {
            let normalized_values = values
                .into_iter()
                .map(|value| GazetteerEntityValue {
                    raw_value: normalize(value),
                    resolved_value: value.to_string(),
                })
                .collect();
            (entity.clone(), normalized_values)
        })
        .collect();

    for (entity, new_entity_values) in normalized_entity_values {
        info!("Injecting values for entity '{}'", entity);

        let parser_dir = &parsers_dirs[&entity];
        println!("injecting: {:?} for {:?} in {:?}", entity, new_entity_values, parser_dir);
        let mut gazetteer_parser = GazetterEntityParser::from_folder(parser_dir)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("could not load gazetteer parser in {:?}", parser_dir)
            })?;

        gazetteer_parser.inject_new_values(new_entity_values, true, from_vanilla)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("could not inject values for entity '{}'", entity)
            })?;

        let parsed = gazetteer_parser.run("jazzy jazzy")
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError { msg: "".to_string() })?;
        println!("parsed: {:?}", parsed);

        fs::remove_dir_all(parser_dir.clone())
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("could not remove previous parser at {:?}", parser_dir)
            })?;

        gazetteer_parser.dump(&parser_dir)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("failed to dump gazetteer parser in {:?}", parser_dir)
            })?;
    }

    info!("Injection performed with success !");
    Ok(())
}

fn get_entity_parsers_dirs<P: AsRef<Path>>(
    nlu_engine_dir: P,
    entity_values: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, PathBuf>, NluInjectionError> {
    let engine_metadata = get_nlu_engine_metadata(nlu_engine_dir.as_ref())?;
    let builtin_parser_info = get_builtin_parser_info(&engine_metadata.builtin_entity_parser_dir)?;
    let custom_parser_info = get_custom_parser_info(&engine_metadata.custom_entity_parser_dir)?;

    entity_values.keys()
        .map(|entity| {
            let parser_info = if engine_metadata.custom_entities.contains(entity) {
                Ok(&custom_parser_info)
            } else if BuiltinGazetteerEntityKind::from_identifier(entity).is_ok() {
                builtin_parser_info
                    .as_ref()
                    .ok_or_else(|| {
                        let msg = format!("could not find gazetteer entity '{}' in engine", entity);
                        NluInjectionErrorKind::EntityNotInjectable { msg }
                    })
            } else if GrammarEntityKind::from_identifier(entity).is_ok() {
                let msg = format!("Entity injection is not allowed for grammar entities: '{}'", entity);
                Err(NluInjectionErrorKind::EntityNotInjectable { msg })
            } else {
                let msg = format!("Unknown entity: '{}'", entity);
                Err(NluInjectionErrorKind::EntityNotInjectable { msg })
            }?;
            let dir: Result<PathBuf, NluInjectionError> = parser_info
                .gazetteer_parser_metadata.parsers_metadata
                .iter()
                .find(|metadata| metadata.entity_identifier == *entity)
                .map(|metadata| parser_info.gazetteer_parser_dir.join(&metadata.entity_parser))
                .ok_or_else(|| {
                    let msg = format!("could not find entity '{}' in engine", entity);
                    NluInjectionErrorKind::EntityNotInjectable { msg }.into()
                });

            Ok((entity.clone(), dir?))
        })
        .collect::<Result<HashMap<String, PathBuf>, NluInjectionError>>()
}


fn get_builtin_parser_info(
    builtin_parser_dir: &PathBuf
) -> Result<Option<GazetteerParserInfo>, NluInjectionError> {
    let builtin_entity_parser_metadata_path = builtin_parser_dir.join("metadata.json");
    let builtin_entity_parser_metadata_file = fs::File::open(&builtin_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("could not open builtin entity parser metadata file in {:?}",
                         builtin_entity_parser_metadata_path)
        })?;
    let builtin_parser_metadata: BuiltinParserMetadata =
        ::serde_json::from_reader(builtin_entity_parser_metadata_file)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("invalid builtin entity parser metadata format in {:?}",
                             builtin_entity_parser_metadata_path)
            })?;
    if let Some(gazetteer_parser_dir) = builtin_parser_metadata.gazetteer_parser
        .map(|directory_name| builtin_parser_dir.join(directory_name)) {
        let gazetteer_parser_metadata_path = builtin_parser_dir
            .join(&gazetteer_parser_dir)
            .join("metadata.json");
        let gazetteer_parser_metadata_file =
            fs::File::open(&gazetteer_parser_metadata_path)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not open gazettteer parser metadata file in {:?}",
                                 gazetteer_parser_metadata_path)
                })?;
        let gazetteer_parser_metadata: GazetteerParserMetadata =
            ::serde_json::from_reader(gazetteer_parser_metadata_file)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("invalid gazetteer parser metadata format in {:?}",
                                 gazetteer_parser_metadata_path)
                })?;
        Ok(Some(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata }))
    } else {
        Ok(None)
    }
}

fn get_custom_parser_info(
    custom_parser_dir: &PathBuf
) -> Result<GazetteerParserInfo, NluInjectionError> {
    let custom_entity_parser_metadata_path = custom_parser_dir.join("metadata.json");
    let custom_entity_parser_metadata_file = fs::File::open(&custom_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("could not open custom entity parser metadata file in {:?}", custom_entity_parser_metadata_path)
        })?;
    let custom_parser_metadata: CustomEntityParserMetadata = ::serde_json::from_reader(
        custom_entity_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("invalid custom entity parser metadata format in {:?}", custom_entity_parser_metadata_path)
        })?;
    let gazetteer_parser_dir = custom_parser_dir.join(custom_parser_metadata.parser_directory);
    let gazetteer_parser_metadata_path = gazetteer_parser_dir.join("metadata.json");
    let gazetteer_parser_metadata_file = fs::File::open(&gazetteer_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("could not open gazetteer parser metadata file in {:?}", gazetteer_parser_metadata_path)
        })?;
    let gazetteer_parser_metadata: GazetteerParserMetadata = ::serde_json::from_reader(
        gazetteer_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("invalid gazetteer parser metadata format in {:?}", gazetteer_parser_metadata_path)
        })?;
    Ok(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata })
}

fn get_nlu_engine_metadata<P: AsRef<Path>>(engine_dir: P) -> Result<NLUEngineInfo, NluInjectionError> {
    let engine_dataset_metadata_path = engine_dir.as_ref().join("nlu_engine.json");
    let config_file = fs::File::open(engine_dataset_metadata_path.clone())
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("could not open nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;
    let nlu_engine_model: NluEngineModel = ::serde_json::from_reader(config_file)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("invalid nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;

    let custom_entities = HashSet::from_iter(
        nlu_engine_model.dataset_metadata.entities.keys().map(|k| k.clone()));

    let builtin_entity_parser_dir = engine_dir.as_ref().join(nlu_engine_model.builtin_entity_parser);
    let custom_entity_parser_dir = engine_dir.as_ref().join(nlu_engine_model.custom_entity_parser);

    Ok(NLUEngineInfo {
        builtin_entity_parser_dir,
        custom_entity_parser_dir,
        custom_entities,
    })
}

#[cfg(test)]
mod tests {
    extern crate mio_httpc;
    extern crate tempfile;
    extern crate zip;

    use self::mio_httpc::CallBuilder;
    use self::tempfile::tempdir;
    use snips_nlu_ontology::*;
    use SnipsNluEngine;
    use utils::extract_nlu_engine_zip_archive;
    use std::iter::FromIterator;
    use std::io::Write;
    use super::*;

    #[test]
    fn test_should_inject() {
        let tdir = tempdir().unwrap();
        let (_, body) = CallBuilder::get().max_response(20000000)
            .timeout_ms(60000)
            .url("https://s3.amazonaws.com/snips/nlu-lm/test/nlu-injection/nlu_engine-0.17.0.zip")
            .unwrap()
            .exec()
            .unwrap();

        let nlu_engine_path = tdir.as_ref().join("nlu_engine.zip");
        let mut nlu_engine_file = fs::File::create(&nlu_engine_path).unwrap();
        nlu_engine_file.write(&body).unwrap();
        let file_reader = fs::File::open(nlu_engine_path).unwrap();

        let engine_dir = extract_nlu_engine_zip_archive(file_reader, tdir.as_ref()).unwrap();

        // Behaviour before injection
        let nlu_engine = SnipsNluEngine::from_path(engine_dir.clone()).unwrap();

        let parsing = nlu_engine.parse("je veux écouter une chanson de artist 1", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je veux écouter une chanson de artist 2", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je souhaiterais écouter l'album thisisthebestalbum", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je voudrais ecouter ma playlist jazzy jazzy", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);

        // values to inject
        let values_as_vec = vec![
            (
                "snips/musicArtist".to_string(),
                vec!["Artist 1".to_string(), "Artist 2".to_string()],
            ),
            (
                "snips/musicAlbum".to_string(),
                vec!["Thisisthebestalbum".to_string()],
            ),
            (
                "playlist".to_string(),
                vec!["jazzy jazzy".to_string()],
            )
        ];
        let values = HashMap::from_iter(values_as_vec);

        // perform injection
        inject_entity_values(&engine_dir, &values, true).unwrap();
        let nlu_engine = SnipsNluEngine::from_path(engine_dir).unwrap();

        // Behavior after injection
        let parsing = nlu_engine.parse("je veux écouter une chanson de artist 1", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic".to_string());
        let ground_true_slots = Some(vec![
            Slot {
                raw_value: "artist 1".to_string(),
                range: Some(31..39),
                entity: "snips/musicArtist".to_string(),
                slot_name: "musicArtist".to_string(),
                value: SlotValue::MusicArtist(StringValue::from("Artist 1")),
            }
        ]);
        assert_eq!(parsing.slots, ground_true_slots);

        let parsing = nlu_engine.parse("je veux écouter une chanson de artist 2", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic".to_string());
        let ground_true_slots = Some(vec![
            Slot {
                raw_value: "artist 2".to_string(),
                range: Some(31..39),
                entity: "snips/musicArtist".to_string(),
                slot_name: "musicArtist".to_string(),
                value: SlotValue::MusicArtist(StringValue::from("Artist 2")),
            }
        ]);
        assert_eq!(parsing.slots, ground_true_slots);

        let parsing = nlu_engine.parse("je souhaiterais écouter l'album thisisthebestalbum", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic".to_string());
        let ground_true_slots = Some(vec![
            Slot {
                raw_value: "thisisthebestalbum".to_string(),
                range: Some(32..50),
                entity: "snips/musicAlbum".to_string(),
                slot_name: "musicAlbum".to_string(),
                value: SlotValue::MusicAlbum(StringValue::from("Thisisthebestalbum")),
            }
        ]);
        assert_eq!(parsing.slots, ground_true_slots);

        let parsing = nlu_engine.parse("je voudrais ecouter ma playlist jazzy jazzy", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic".to_string());
        let ground_true_slots = Some(vec![
            Slot {
                raw_value: "the best playlist jazzy jazzy".to_string(),
                range: Some(32..53),
                entity: "playlist".to_string(),
                slot_name: "playlist".to_string(),
                value: SlotValue::Custom(StringValue::from("jazzy jazzy")),
            }
        ]);
        assert_eq!(parsing.slots, ground_true_slots);
    }
}
