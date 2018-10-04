use entity_parser::custom_entity_parser::CustomEntityParserMetadata;
use failure::ResultExt;
use injection::errors::{NluInjectionError, NluInjectionErrorKind};
use models::nlu_engine::NluEngineModel;
use snips_nlu_ontology::Language;
use snips_nlu_ontology_parsers::{BuiltinParserMetadata, GazetteerParserMetadata};
use snips_nlu_ontology_parsers::{GazetteerEntityParser as _GazetterEntityParser, GazetteerEntityValue};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::iter::FromIterator;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

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
pub fn inject_gazetteer_entity_parser_values<P: AsRef<Path>>(
    nlu_engine_dir: P,
    entity_values: &HashMap<InjectedEntity, Vec<InjectedValue>>,
    from_vanilla: bool,
) -> Result<(), NluInjectionError> {
    info!("Starting injection...");

    info!("Retrieving parsers paths...");
    // Get NLU engine metadata
    let engine_metadata = get_nlu_engine_metadata(nlu_engine_dir.as_ref())?;

    let builtin_parser_info = get_builtin_parser_info(&engine_metadata.builtin_entity_parser_dir)?;
    let custom_parser_info = get_custom_parser_info(&engine_metadata.custom_entity_parser_dir)?;

    // Get gazetteer parsers paths
    let parsers_dirs: HashMap<InjectedEntity, PathBuf> = entity_values.keys()
        .map(|entity| {
            let dir = get_gazetteer_parser_dir(
                &engine_metadata,
                &builtin_parser_info,
                &custom_parser_info,
                entity,
            )?;
            Ok((entity.clone(), dir))
        })
        .collect::<Result<_, NluInjectionError>>()?;

    info!("Normalizing injected values...");
    // Update all values
    let normalized_entity_values: HashMap<InjectedEntity, Vec<GazetteerEntityValue>> = entity_values
        .into_iter()
        .map(|(entity, values)| {
            let normalized_values: Vec<GazetteerEntityValue> = values
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
        info!("Injecting injecting in {}", entity);

        let parser_dir = &parsers_dirs[&entity];
        println!("injecting: {:?} for {:?} in {:?}", entity, new_entity_values,  parser_dir);
        let mut gazetteer_parser = _GazetterEntityParser::from_folder(parser_dir)
            .with_context(|_| NluInjectionErrorKind::GazetteerParserInjectionError {
                msg: format!("could not load gazetteer parser in {:?}", parser_dir)
            })?;

        gazetteer_parser.inject_new_values(new_entity_values, true, from_vanilla)
            .with_context(|_| NluInjectionErrorKind::GazetteerParserInjectionError {
                msg: format!("could not inject in {} gazetteer parser", entity)
            })?;

        let parsed = gazetteer_parser.run("jazzy jazzy").with_context(|_| NluInjectionErrorKind::Io {msg: "".to_string()})?;
        println!("parsed: {:?}", parsed);

        fs::remove_dir_all(parser_dir.clone())
            .with_context(|_| NluInjectionErrorKind::Io {
                msg: format!("could not remove previous parser at {:?}", parser_dir)
            })?;

        gazetteer_parser.dump(&parser_dir)
            .with_context(|_| NluInjectionErrorKind::GazetteerParserInjectionError {
                msg: format!("failed to dump gazetteer parser in {:?}", parser_dir)
            })?;
    }

    info!("Injection performed with success !");
    Ok(())
}


fn get_builtin_parser_info(
    builtin_parser_dir: &PathBuf
) -> Result<GazetteerParserInfo, NluInjectionError> {
    let builtin_entity_parser_metadata_path = builtin_parser_dir.join("metadata.json");
    let builtin_entity_parser_metadata_file = fs::File::open(&builtin_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("could not open builtin entity parser metadata file in {:?}", builtin_entity_parser_metadata_path)
        })?;
    let builtin_parser_metadata: BuiltinParserMetadata = ::serde_json::from_reader(
        builtin_entity_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("invalid builtin entity parser metadata format in {:?}", builtin_entity_parser_metadata_path)
        })?;
    let gazetteer_parser_dir = builtin_parser_metadata.gazetteer_parser
        .ok_or(NluInjectionErrorKind::NotInjectableEntity {
            msg: "NLU engine has no gazetteer parser in its builtin entity parser".to_string()
        })?;
    let gazetteer_parser_dir = builtin_parser_dir
        .join(gazetteer_parser_dir);
    let gazetteer_parser_metadata_path = gazetteer_parser_dir
        .join("metadata.json");
    let gazetteer_parser_metadata_file = fs::File::open(&gazetteer_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("could not open gazettteer parser metadata file in {:?}", gazetteer_parser_metadata_path)
        })?;
    let gazetteer_parser_metadata: GazetteerParserMetadata = ::serde_json::from_reader(
        gazetteer_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("invalid gazetteer parser metadata format in {:?}", gazetteer_parser_metadata_path)
        })?;
    Ok(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata })
}

fn get_custom_parser_info(
    custom_parser_dir: &PathBuf
) -> Result<GazetteerParserInfo, NluInjectionError> {
    let custom_entity_parser_metadata_path = custom_parser_dir.join("metadata.json");
    let custom_entity_parser_metadata_file = fs::File::open(&custom_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("could not open custom entity parser metadata file in {:?}", custom_entity_parser_metadata_path)
        })?;
    let custom_parser_metadata: CustomEntityParserMetadata = ::serde_json::from_reader(
        custom_entity_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("invalid custom entity parser metadata format in {:?}", custom_entity_parser_metadata_path)
        })?;
    let gazetteer_parser_dir = custom_parser_metadata.parser_directory;
    let gazetteer_parser_dir = custom_parser_dir
        .join(gazetteer_parser_dir);
    let gazetteer_parser_metadata_path = gazetteer_parser_dir
        .join("metadata.json");
    let gazetteer_parser_metadata_file = fs::File::open(&gazetteer_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("could not open gazetteer parser metadata file in {:?}", gazetteer_parser_metadata_path)
        })?;
    let gazetteer_parser_metadata: GazetteerParserMetadata = ::serde_json::from_reader(
        gazetteer_parser_metadata_file)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("invalid gazetteer parser metadata format in {:?}", gazetteer_parser_metadata_path)
        })?;
    Ok(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata })
}

fn get_gazetteer_parser_dir(
    engine_info: &NLUEngineInfo,
    builtin_parser_info: &GazetteerParserInfo,
    custom_parser_info: &GazetteerParserInfo,
    entity: &InjectedEntity,
) -> Result<PathBuf, NluInjectionError> {
    let parser_path = if engine_info.custom_entities.contains(entity) {
        let mut maybe_parser_name = None;
        for metadata in custom_parser_info.gazetteer_parser_metadata.parsers_metadata.iter() {
            if metadata.entity_identifier == *entity {
                maybe_parser_name = Some(&metadata.entity_parser);
                break;
            }
        }
        let parser_name = maybe_parser_name.ok_or(
            NluInjectionErrorKind::NotInjectableEntity {
                msg: format!("could find entity {} in engine", entity)
            })?;
        custom_parser_info.gazetteer_parser_dir.join(parser_name)
    } else {
        let mut maybe_parsername = None;
        for metadata in builtin_parser_info.gazetteer_parser_metadata.parsers_metadata.iter() {
            if metadata.entity_identifier == *entity {
                maybe_parsername = Some(&metadata.entity_parser);
                break;
            }
        }
        let parser_name = maybe_parsername.ok_or(
            NluInjectionErrorKind::NotInjectableEntity {
                msg: format!("could find entity {} in engine", entity)
            })?;
        builtin_parser_info.gazetteer_parser_dir.join(parser_name)
    };
    Ok(parser_path)
}

fn get_nlu_engine_metadata<P: AsRef<Path>>(engine_dir: P) -> Result<NLUEngineInfo, NluInjectionError> {
    let engine_dataset_metadata_path = engine_dir.as_ref().join("nlu_engine.json");
    let config_file = fs::File::open(engine_dataset_metadata_path.clone())
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("could not open nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;
    let nlu_engine_model: NluEngineModel = ::serde_json::from_reader(config_file)
        .with_context(|_| NluInjectionErrorKind::Io {
            msg: format!("invalid nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;

    // Check the engine language
    let language_code = nlu_engine_model.dataset_metadata.language_code;
    Language::from_str(&*language_code)
        .with_context(|_| NluInjectionErrorKind::NotInjectableEntity {
            msg: format!("{} language is not supported", language_code)
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
    use self::zip::ZipArchive;
    use snips_nlu_ontology::*;
    use SnipsNluEngine;
    use std::io;
    use std::io::Write;
    use std::iter::FromIterator;
    use std::path::Component;
    use super::*;

    #[test]
    fn test_should_inject() {
        let tdir = tempdir().unwrap();
        let (_, body) = CallBuilder::get().max_response(20000000)
            .timeout_ms(60000)
            .url("https://s3.amazonaws.com/snips/nlu-lm/test/nlu-injection/nlu_engine.zip")
            .unwrap()
            .exec()
            .unwrap();
        let nlu_engine_path = tdir.as_ref().join("nlu_engine.zip");
        let mut nlu_engine_file = fs::File::create(&nlu_engine_path).unwrap();
        nlu_engine_file.write(&body).unwrap();

        let mut archive = ZipArchive::new(fs::File::open(&nlu_engine_path).unwrap()).unwrap();
        let temp_dir = tempfile::Builder::new()
            .prefix("temp_dir_nlu_")
            .tempdir().unwrap();
        let temp_dir_path = temp_dir.path();

        for file_index in 0..archive.len() {
            let mut file = archive.by_index(file_index).unwrap();
            let outpath = temp_dir_path.join(file.sanitized_name());

            if (&*file.name()).ends_with('/') || (&*file.name()).ends_with('\\') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        let first_archive_file = archive
            .by_index(0).unwrap()
            .sanitized_name();

        let engine_dir_path = first_archive_file
            .components()
            .find(|component| if let Component::Normal(_) = component { true } else { false })
            .ok_or_else(|| format_err!("Trained engine archive is incorrect")).unwrap()
            .as_os_str();
        let engine_dir_name = engine_dir_path
            .to_str()
            .ok_or_else(|| format_err!("Engine directory name is empty")).unwrap();

        let final_engine_dir = temp_dir_path.join(engine_dir_name);

        // Behaviour before injection
        let nlu_engine = SnipsNluEngine::from_path(final_engine_dir.clone()).unwrap();

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
        let values_as_vec: Vec<(String, Vec<String>)> = vec![
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
        inject_gazetteer_entity_parser_values(&final_engine_dir, &values, true).unwrap();
        let nlu_engine = SnipsNluEngine::from_path(final_engine_dir).unwrap();

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
