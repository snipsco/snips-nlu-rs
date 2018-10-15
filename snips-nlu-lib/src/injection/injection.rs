use std::collections::{HashMap, HashSet};
use std::fs;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use failure::ResultExt;
use itertools::Itertools;
use nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_ontology::{BuiltinGazetteerEntityKind, GrammarEntityKind};
use snips_nlu_ontology_parsers::{BuiltinParserMetadata, GazetteerParserMetadata};
use snips_nlu_ontology_parsers::gazetteer_entity_parser::{EntityValue as GazetteerEntityValue,
                                                          Parser as GazetteerEntityParser};
use snips_nlu_utils::token::tokenize_light;

use entity_parser::custom_entity_parser::CustomEntityParserMetadata;
use entity_parser::custom_entity_parser::CustomEntityParserUsage;
use injection::errors::{NluInjectionError, NluInjectionErrorKind};
use models::nlu_engine::NluEngineModel;
use nlu_engine::load_engine_shared_resources;
use resources::stemmer::Stemmer;
use resources::SharedResources;

pub type InjectedEntity = String;
pub type InjectedValue = String;

fn normalize(s: &str) -> String {
    s.to_lowercase()
}

struct NluEngineInfo {
    language: NluUtilsLanguage,
    builtin_entity_parser_dir: PathBuf,
    custom_entity_parser_dir: PathBuf,
    custom_entities: HashSet<InjectedEntity>,
}

struct GazetteerParserInfo {
    gazetteer_parser_dir: PathBuf,
    gazetteer_parser_metadata: GazetteerParserMetadata,
    parser_usage: Option<CustomEntityParserUsage>,
}


pub struct NluInjector<P: AsRef<Path>> {
    nlu_engine_dir: P,
    entity_values: HashMap<InjectedEntity, Vec<InjectedValue>>,
    from_vanilla: bool,
    shared_resources: Option<Arc<SharedResources>>,
}

impl<P: AsRef<Path>> NluInjector<P> {
    pub fn new(nlu_engine_dir: P) -> Self {
        Self {
            nlu_engine_dir,
            entity_values: HashMap::new(),
            from_vanilla: false,
            shared_resources: None,
        }
    }

    pub fn add_value(mut self, entity: InjectedEntity, value: InjectedValue) -> Self {
        self.entity_values
            .entry(entity)
            .or_insert(vec![])
            .push(value);
        self
    }

    pub fn from_vanilla(mut self, from_vanilla: bool) -> Self {
        self.from_vanilla = from_vanilla;
        self
    }

    pub fn shared_resources(mut self, shared_resources: Arc<SharedResources>) -> Self {
        self.shared_resources = Some(shared_resources);
        self
    }

    pub fn inject(self) -> Result<(), NluInjectionError> {
        info!("Starting injection...");

        info!("Retrieving parsers paths...");
        let engine_info = get_nlu_engine_info(self.nlu_engine_dir.as_ref())?;
        let builtin_parser_info = get_builtin_parser_info(
            &engine_info.builtin_entity_parser_dir)?;
        let custom_parser_info = get_custom_parser_info(
            &engine_info.custom_entity_parser_dir)?;
        let parsers_dirs = get_entity_parsers_dirs(
            &engine_info,
            &builtin_parser_info,
            &custom_parser_info,
            &self.entity_values,
        )?;

        let shared_resources = if let Some(resources) = self.shared_resources {
            Ok(resources)
        } else {
            load_engine_shared_resources(self.nlu_engine_dir.as_ref())
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                        msg: format!("Could not load shared resources from {:?}", self.nlu_engine_dir.as_ref())
                    })
        }?;

        let maybe_stemmer = shared_resources.stemmer.as_ref();

        info!("Normalizing injected values...");
        // Update all values
        let normalized_entity_values = normalize_entity_values(
            &self.entity_values, engine_info, custom_parser_info, maybe_stemmer)?;

        for (entity, new_entity_values) in normalized_entity_values {
            info!("Injecting values for entity '{}'", entity);

            let parser_dir = &parsers_dirs[&entity];
            let mut gazetteer_parser = GazetterEntityParser::from_folder(parser_dir)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not load gazetteer parser in {:?}", parser_dir)
                })?;

            gazetteer_parser.inject_new_values(new_entity_values, true, self.from_vanilla)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!("could not inject values for entity '{}'", entity)
            })?;

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
}


fn get_entity_parsers_dirs(
    engine_info: &NluEngineInfo,
    builtin_parser_info: &Option<GazetteerParserInfo>,
    custom_parser_info: &GazetteerParserInfo,
    entity_values: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, PathBuf>, NluInjectionError> {
    entity_values.keys()
        .map(|entity| {
            let parser_info = if engine_info.custom_entities.contains(entity) {
                Ok(custom_parser_info)
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
        let gazetteer_parser_metadata_path = gazetteer_parser_dir
            .join("metadata.json");
        let gazetteer_parser_metadata_file =
            fs::File::open(&gazetteer_parser_metadata_path)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not open gazetteer parser metadata file in {:?}",
                                 gazetteer_parser_metadata_path)
                })?;
        let gazetteer_parser_metadata: GazetteerParserMetadata =
            ::serde_json::from_reader(gazetteer_parser_metadata_file)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("invalid gazetteer parser metadata format in {:?}",
                                 gazetteer_parser_metadata_path)
                })?;
        Ok(Some(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata, parser_usage: None }))
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
    let parser_usage = CustomEntityParserUsage::from_u8(custom_parser_metadata.parser_usage)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: "found invalid parser usage in custom entity parser".to_string()
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
    Ok(GazetteerParserInfo { gazetteer_parser_dir, gazetteer_parser_metadata, parser_usage: Some(parser_usage) })
}

fn get_nlu_engine_info<P: AsRef<Path>>(engine_dir: P) -> Result<NluEngineInfo, NluInjectionError> {
    let engine_dataset_metadata_path = engine_dir.as_ref().join("nlu_engine.json");
    let config_file = fs::File::open(engine_dataset_metadata_path.clone())
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("could not open nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;
    let nlu_engine_model: NluEngineModel = ::serde_json::from_reader(config_file)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!("invalid nlu engine model file in {:?}", engine_dataset_metadata_path)
        })?;

    let language = NluUtilsLanguage::from_str(&*nlu_engine_model.dataset_metadata.language_code)
        .map_err(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: "invalid nlu engine language".to_string()
        })?;

    let custom_entities = HashSet::from_iter(
        nlu_engine_model.dataset_metadata.entities.keys().map(|k| k.clone()));

    let builtin_entity_parser_dir = engine_dir.as_ref().join(nlu_engine_model.builtin_entity_parser);
    let custom_entity_parser_dir = engine_dir.as_ref().join(nlu_engine_model.custom_entity_parser);

    Ok(NluEngineInfo {
        language,
        builtin_entity_parser_dir,
        custom_entity_parser_dir,
        custom_entities,
    })
}

fn normalize_entity_values(
    entity_values: &HashMap<String, Vec<InjectedValue>>,
    engine_info: NluEngineInfo,
    custom_parser_info: GazetteerParserInfo,
    maybe_stemmer: Option<&Arc<Stemmer>>,
) -> Result<HashMap<String, Vec<GazetteerEntityValue>>, NluInjectionError> {
    let parser_usage = custom_parser_info.parser_usage
        .ok_or_else(|| {
            let msg =  "custom parser has no parser usage".to_string();
                        NluInjectionErrorKind::InternalInjectionError { msg }
        })?;

    entity_values
        .iter()
        .map(|(entity, values)| {
            let normalized_values: Vec<GazetteerEntityValue> = values
                .into_iter()
                .map(|value| GazetteerEntityValue {
                    raw_value: normalize(value),
                    resolved_value: value.to_string(),
                })
                .collect();
            let normalized_stemmed_values: Result<Vec<GazetteerEntityValue>, NluInjectionError> = if engine_info
                .custom_entities
                .contains(entity) {
                let mut original_value: Vec<GazetteerEntityValue> = match parser_usage {
                    CustomEntityParserUsage::WithStems => vec![],
                    _ => normalized_values.clone()
                };

                let stemmed_values: Result<Vec<GazetteerEntityValue>, NluInjectionError> = match parser_usage {
                    CustomEntityParserUsage::WithoutStems => Ok(vec![]),
                    _ => {
                        let stemmed: Vec<GazetteerEntityValue> = normalized_values
                            .into_iter()
                            .map(|value|{
                                let stemmer = &maybe_stemmer
                                    .ok_or(NluInjectionErrorKind::InternalInjectionError {
                                        msg: format!("found {:?} parser usage but no stemmer in NLU engine.", parser_usage)
                                    })?;
                                let raw_value = tokenize_light(
                                    &*value.raw_value, engine_info.language)
                                    .into_iter()
                                    .map(|token| stemmer.stem(&*token))
                                    .join(" ");

                                Ok(GazetteerEntityValue {
                                    raw_value,
                                    resolved_value: value.resolved_value,
                                })
                            })
                            .collect::<Result<_, NluInjectionError>>()?;
                        Ok(stemmed)
                    }
                };

                original_value.extend(stemmed_values?);
                Ok(original_value)
            } else {
                Ok(normalized_values)
            };

            Ok((entity.clone(), normalized_stemmed_values?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    extern crate fs_extra;

    use self::fs_extra::dir;
    use self::tempfile::tempdir;
    use SharedResources;
    use snips_nlu_ontology::*;
    use SnipsNluEngine;

    use super::*;
    use testutils::file_path;

    #[derive(Clone)]
    struct MockedStemmer<'a>{
        values: HashMap<&'a str, &'a str>,
    }

    impl<'a> Stemmer for MockedStemmer<'a> {
        fn stem(&self, value: &str) -> String {
            let stemmed = self.values
                .get(value)
                .map(|stemmed_value| *stemmed_value)
                .unwrap_or(value)
                .to_string();
            stemmed
        }
    }

    #[test]
    fn test_should_inject() {
        let path = file_path("tests")
            .join("models")
            .join("nlu_engine_music");

        let tdir = tempdir().unwrap();
        dir::copy(path, tdir.as_ref(), &dir::CopyOptions::new()).unwrap();
        let engine_dir = tdir.as_ref().join("nlu_engine_music");
        let engine_shared_resources = load_engine_shared_resources(
            &engine_dir)
            .unwrap();

        let stems = vec![("jazzy", "jazz")]
            .into_iter()
            .collect();
        let stemmer = MockedStemmer { values: stems };
        let mocked_resources = Arc::new(
                SharedResources {
            builtin_entity_parser: engine_shared_resources.as_ref().builtin_entity_parser.clone(),
            custom_entity_parser: engine_shared_resources.as_ref().custom_entity_parser.clone(),
            gazetteers: engine_shared_resources.as_ref().gazetteers.clone(),
            stemmer: Some(Arc::new(stemmer.clone())),
            word_clusterers: HashMap::new(),
        });

        // Behaviour before injection
        let nlu_engine = SnipsNluEngine::from_path_with_resources(
            &engine_dir, mocked_resources.clone()).unwrap();
        let parsing = nlu_engine.parse("je veux ecouter une chanson de artist 1 please", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je veux ecouter une chanson de artist 2  please", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je souhaiterais écouter l'album thisisthebestalbum", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);
        let parsing = nlu_engine.parse("je voudrais ecouter ma playlist jazz jazz", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic");
        assert_eq!(parsing.slots.unwrap(), vec![]);

        // values to inject
        let values = vec![
            (
                "snips/musicArtist".to_string(),
                "Artist 1".to_string(),
            ),
            (
                "snips/musicArtist".to_string(),
                "Artist 2".to_string(),
            ),
            (
                "snips/musicAlbum".to_string(),
                "Thisisthebestalbum".to_string(),
            ),
            (
                "playlist".to_string(),
                "jazzy jazzy".to_string(),
            )
        ];

        let mut injector = NluInjector::new(&engine_dir)
            .from_vanilla(true)
            .shared_resources(mocked_resources);

        for (entity, value) in values {
            injector = injector.add_value(entity, value);
        }

        injector.inject().unwrap();

        let injected_resources = load_engine_shared_resources(&engine_dir)
            .unwrap();

        let mocked_injected_resources = SharedResources {
            builtin_entity_parser: injected_resources.builtin_entity_parser.clone(),
            custom_entity_parser: injected_resources.custom_entity_parser.clone(),
            gazetteers: injected_resources.gazetteers.clone(),
            stemmer: Some(Arc::new(stemmer)),
            word_clusterers: injected_resources.word_clusterers.clone(),
        };

        let nlu_engine = SnipsNluEngine::from_path_with_resources(
            &engine_dir, Arc::new(mocked_injected_resources)).unwrap();

        // Behavior after injection
        let parsing = nlu_engine.parse("je veux ecouter une chanson de artist 1 please", None).unwrap();
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

        let parsing = nlu_engine.parse("je veux ecouter une chanson de artist 2 please", None).unwrap();
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

        let parsing = nlu_engine.parse("je voudrais ecouter ma playlist jazz jazz", None).unwrap();
        assert_eq!(parsing.intent.unwrap().intent_name, "adri:PlayMusic".to_string());
        let ground_true_slots = Some(vec![
            Slot {
                raw_value: "jazz jazz".to_string(),
                range: Some(32..41),
                entity: "playlist".to_string(),
                slot_name: "playlist".to_string(),
                value: SlotValue::Custom(StringValue::from("jazzy jazzy")),
            }
        ]);
        assert_eq!(parsing.slots, ground_true_slots);
    }
}
