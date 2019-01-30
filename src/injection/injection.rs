use std::collections::{HashMap, HashSet};
use std::fs;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use failure::ResultExt;
use itertools::Itertools;
use log::info;
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_ontology::{BuiltinGazetteerEntityKind, GrammarEntityKind};
use snips_nlu_parsers::gazetteer_entity_parser::{
    EntityValue as GazetteerEntityValue, Parser as GazetteerEntityParser,
};
use snips_nlu_parsers::{BuiltinParserMetadata, GazetteerParserMetadata};
use snips_nlu_utils::token::tokenize_light;

use crate::entity_parser::custom_entity_parser::CustomEntityParserMetadata;
use crate::entity_parser::custom_entity_parser::CustomEntityParserUsage;
use crate::models::nlu_engine::NluEngineModel;
use crate::resources::loading::load_engine_shared_resources;
use crate::resources::stemmer::Stemmer;
use crate::resources::SharedResources;

use super::errors::{NluInjectionError, NluInjectionErrorKind};

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

struct BuiltinGazetteerParserInfo {
    gazetteer_parser_dir: PathBuf,
    gazetteer_parser_metadata: GazetteerParserMetadata,
}

struct CustomGazetteerParserInfo {
    gazetteer_parser_dir: PathBuf,
    gazetteer_parser_metadata: GazetteerParserMetadata,
    parser_usage: CustomEntityParserUsage,
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
            .or_insert_with(|| vec![])
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
        let builtin_parser_info = get_builtin_parser_info(&engine_info.builtin_entity_parser_dir)?;
        let custom_parser_info = get_custom_parser_info(&engine_info.custom_entity_parser_dir)?;
        let parsers_dirs = get_entity_parsers_dirs(
            &engine_info,
            &builtin_parser_info,
            &custom_parser_info,
            &self.entity_values,
        )?;

        let shared_resources = if let Some(resources) = self.shared_resources {
            Ok(resources)
        } else {
            load_engine_shared_resources(self.nlu_engine_dir.as_ref()).with_context(|_| {
                NluInjectionErrorKind::InternalInjectionError {
                    msg: format!(
                        "Could not load shared resources from {:?}",
                        self.nlu_engine_dir.as_ref()
                    ),
                }
            })
        }?;

        let maybe_stemmer = shared_resources.stemmer.as_ref();

        // Normalize and stem all values if needed
        info!("Normalizing injected values...");
        let normalized_entity_values = self
            .entity_values
            .into_iter()
            .map(|(entity, values)| {
                let normalize_entity_values = normalize_entity_value(values);
                if engine_info.custom_entities.contains(&*entity) {
                    let stemmed_entity_values = stem_entity_value(
                        normalize_entity_values,
                        &engine_info,
                        &custom_parser_info,
                        &maybe_stemmer,
                    )?;
                    Ok((entity, stemmed_entity_values))
                } else {
                    Ok((entity, normalize_entity_values))
                }
            })
            .collect::<Result<HashMap<_, _>, NluInjectionError>>()?;

        for (entity, new_entity_values) in normalized_entity_values {
            info!("Injecting values for entity '{}'", entity);

            let parser_dir = &parsers_dirs[&entity];
            let mut gazetteer_parser = GazetteerEntityParser::from_folder(parser_dir)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not load gazetteer parser in {:?}", parser_dir),
                })?;

            gazetteer_parser
                .inject_new_values(new_entity_values, true, self.from_vanilla)
                .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not inject values for entity '{}'", entity),
                })?;

            fs::remove_dir_all(parser_dir.clone()).with_context(|_| {
                NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("could not remove previous parser at {:?}", parser_dir),
                }
            })?;

            gazetteer_parser.dump(&parser_dir).with_context(|_| {
                NluInjectionErrorKind::InternalInjectionError {
                    msg: format!("failed to dump gazetteer parser in {:?}", parser_dir),
                }
            })?;
        }

        info!("Injection performed with success !");
        Ok(())
    }
}

fn get_entity_parsers_dirs(
    engine_info: &NluEngineInfo,
    maybe_builtin_parser_info: &Option<BuiltinGazetteerParserInfo>,
    custom_parser_info: &CustomGazetteerParserInfo,
    entity_values: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, PathBuf>, NluInjectionError> {
    entity_values
        .keys()
        .map(|entity| {
            let dir = if engine_info.custom_entities.contains(entity) {
                custom_parser_info
                    .gazetteer_parser_metadata
                    .parsers_metadata
                    .iter()
                    .find(|metadata| metadata.entity_identifier == *entity)
                    .map(|metadata| {
                        custom_parser_info
                            .gazetteer_parser_dir
                            .join(&metadata.entity_parser)
                    })
                    .ok_or_else(|| {
                        let msg = format!("could not find entity '{}' in engine", entity);
                        NluInjectionErrorKind::EntityNotInjectable { msg }
                    })
            } else if BuiltinGazetteerEntityKind::from_identifier(entity).is_ok() {
                let builtin_parser_info = maybe_builtin_parser_info.as_ref().ok_or_else(|| {
                    let msg = format!("could not find gazetteer entity '{}' in engine", entity);
                    NluInjectionErrorKind::EntityNotInjectable { msg }
                })?;
                builtin_parser_info
                    .gazetteer_parser_metadata
                    .parsers_metadata
                    .iter()
                    .find(|metadata| metadata.entity_identifier == *entity)
                    .map(|metadata| {
                        builtin_parser_info
                            .gazetteer_parser_dir
                            .join(&metadata.entity_parser)
                    })
                    .ok_or_else(|| {
                        let msg = format!("could not find entity '{}' in engine", entity);
                        NluInjectionErrorKind::EntityNotInjectable { msg }
                    })
            } else if GrammarEntityKind::from_identifier(entity).is_ok() {
                let msg = format!(
                    "Entity injection is not allowed for grammar entities: '{}'",
                    entity
                );
                Err(NluInjectionErrorKind::EntityNotInjectable { msg })
            } else {
                let msg = format!("Unknown entity: '{}'", entity);
                Err(NluInjectionErrorKind::EntityNotInjectable { msg })
            }?;
            Ok((entity.clone(), dir))
        })
        .collect::<Result<HashMap<String, PathBuf>, NluInjectionError>>()
}

fn get_builtin_parser_info(
    builtin_parser_dir: &PathBuf,
) -> Result<Option<BuiltinGazetteerParserInfo>, NluInjectionError> {
    let builtin_entity_parser_metadata_path = builtin_parser_dir.join("metadata.json");
    let builtin_entity_parser_metadata_file = fs::File::open(&builtin_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!(
                "could not open builtin entity parser metadata file in {:?}",
                builtin_entity_parser_metadata_path
            ),
        })?;
    let builtin_parser_metadata: BuiltinParserMetadata =
        serde_json::from_reader(builtin_entity_parser_metadata_file).with_context(|_| {
            NluInjectionErrorKind::InternalInjectionError {
                msg: format!(
                    "invalid builtin entity parser metadata format in {:?}",
                    builtin_entity_parser_metadata_path
                ),
            }
        })?;
    if let Some(gazetteer_parser_dir) = builtin_parser_metadata
        .gazetteer_parser
        .map(|directory_name| builtin_parser_dir.join(directory_name))
    {
        let gazetteer_parser_metadata_path = gazetteer_parser_dir.join("metadata.json");
        let gazetteer_parser_metadata_file = fs::File::open(&gazetteer_parser_metadata_path)
            .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
                msg: format!(
                    "could not open gazetteer parser metadata file in {:?}",
                    gazetteer_parser_metadata_path
                ),
            })?;
        let gazetteer_parser_metadata: GazetteerParserMetadata =
            serde_json::from_reader(gazetteer_parser_metadata_file).with_context(|_| {
                NluInjectionErrorKind::InternalInjectionError {
                    msg: format!(
                        "invalid gazetteer parser metadata format in {:?}",
                        gazetteer_parser_metadata_path
                    ),
                }
            })?;
        let parser_info = BuiltinGazetteerParserInfo {
            gazetteer_parser_dir,
            gazetteer_parser_metadata,
        };
        Ok(Some(parser_info))
    } else {
        Ok(None)
    }
}

fn get_custom_parser_info(
    custom_parser_dir: &PathBuf,
) -> Result<CustomGazetteerParserInfo, NluInjectionError> {
    let custom_entity_parser_metadata_path = custom_parser_dir.join("metadata.json");
    let custom_entity_parser_metadata_file = fs::File::open(&custom_entity_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!(
                "could not open custom entity parser metadata file in {:?}",
                custom_entity_parser_metadata_path
            ),
        })?;
    let custom_parser_metadata: CustomEntityParserMetadata =
        serde_json::from_reader(custom_entity_parser_metadata_file).with_context(|_| {
            NluInjectionErrorKind::InternalInjectionError {
                msg: format!(
                    "invalid custom entity parser metadata format in {:?}",
                    custom_entity_parser_metadata_path
                ),
            }
        })?;

    let gazetteer_parser_dir = custom_parser_dir.join(custom_parser_metadata.parser_directory);
    let gazetteer_parser_metadata_path = gazetteer_parser_dir.join("metadata.json");
    let gazetteer_parser_metadata_file = fs::File::open(&gazetteer_parser_metadata_path)
        .with_context(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: format!(
                "could not open gazetteer parser metadata file in {:?}",
                gazetteer_parser_metadata_path
            ),
        })?;
    let gazetteer_parser_metadata: GazetteerParserMetadata =
        serde_json::from_reader(gazetteer_parser_metadata_file).with_context(|_| {
            NluInjectionErrorKind::InternalInjectionError {
                msg: format!(
                    "invalid gazetteer parser metadata format in {:?}",
                    gazetteer_parser_metadata_path
                ),
            }
        })?;
    let parser_info = CustomGazetteerParserInfo {
        gazetteer_parser_dir,
        gazetteer_parser_metadata,
        parser_usage: custom_parser_metadata.parser_usage,
    };
    Ok(parser_info)
}

fn get_nlu_engine_info<P: AsRef<Path>>(engine_dir: P) -> Result<NluEngineInfo, NluInjectionError> {
    let engine_dataset_metadata_path = engine_dir.as_ref().join("nlu_engine.json");
    let config_file = fs::File::open(engine_dataset_metadata_path.clone()).with_context(|_| {
        NluInjectionErrorKind::InternalInjectionError {
            msg: format!(
                "could not open nlu engine model file in {:?}",
                engine_dataset_metadata_path
            ),
        }
    })?;
    let nlu_engine_model: NluEngineModel =
        serde_json::from_reader(config_file).with_context(|_| {
            NluInjectionErrorKind::InternalInjectionError {
                msg: format!(
                    "invalid nlu engine model file in {:?}",
                    engine_dataset_metadata_path
                ),
            }
        })?;

    let language = NluUtilsLanguage::from_str(&*nlu_engine_model.dataset_metadata.language_code)
        .map_err(|_| NluInjectionErrorKind::InternalInjectionError {
            msg: "invalid nlu engine language".to_string(),
        })?;

    let custom_entities =
        HashSet::from_iter(nlu_engine_model.dataset_metadata.entities.keys().cloned());

    let builtin_entity_parser_dir = engine_dir
        .as_ref()
        .join(nlu_engine_model.builtin_entity_parser);
    let custom_entity_parser_dir = engine_dir
        .as_ref()
        .join(nlu_engine_model.custom_entity_parser);

    Ok(NluEngineInfo {
        language,
        builtin_entity_parser_dir,
        custom_entity_parser_dir,
        custom_entities,
    })
}

fn normalize_entity_value(entity_values: Vec<InjectedValue>) -> Vec<GazetteerEntityValue> {
    entity_values
        .into_iter()
        .map(|value| GazetteerEntityValue {
            raw_value: normalize(&*value),
            resolved_value: value,
        })
        .collect()
}

fn stem_entity_value(
    entity_values: Vec<GazetteerEntityValue>,
    engine_info: &NluEngineInfo,
    custom_entity_parser_info: &CustomGazetteerParserInfo,
    maybe_stemmer: &Option<&Arc<Stemmer>>,
) -> Result<Vec<GazetteerEntityValue>, NluInjectionError> {
    let stemmed_entity_values = match custom_entity_parser_info.parser_usage {
        CustomEntityParserUsage::WithoutStems => vec![],
        _ => entity_values
            .iter()
            .map(|value| {
                let stemmer =
                    &maybe_stemmer.ok_or(NluInjectionErrorKind::InternalInjectionError {
                        msg: format!(
                            "found {:?} parser usage but no stemmer in NLU engine.",
                            custom_entity_parser_info.parser_usage
                        ),
                    })?;
                let raw_value = tokenize_light(&*value.raw_value, engine_info.language)
                    .into_iter()
                    .map(|token| stemmer.stem(&*token))
                    .join(" ");

                Ok(GazetteerEntityValue {
                    raw_value,
                    resolved_value: value.resolved_value.clone(),
                })
            })
            .collect::<Result<_, NluInjectionError>>()?,
    };
    let all_values = match custom_entity_parser_info.parser_usage {
        CustomEntityParserUsage::WithStems => stemmed_entity_values,
        _ => {
            let mut values = entity_values;
            values.extend(stemmed_entity_values);
            values.into_iter().unique().collect::<Vec<_>>()
        }
    };
    Ok(all_values)
}

#[cfg(test)]
mod tests {
    extern crate fs_extra;
    extern crate tempfile;

    use self::fs_extra::dir;
    use self::tempfile::tempdir;
    use snips_nlu_ontology::*;

    use crate::SharedResources;
    use crate::SnipsNluEngine;

    use super::*;

    #[derive(Clone)]
    struct MockedStemmer<'a> {
        values: HashMap<&'a str, &'a str>,
    }

    impl<'a> Stemmer for MockedStemmer<'a> {
        fn stem(&self, value: &str) -> String {
            let stemmed = self
                .values
                .get(value)
                .map(|stemmed_value| *stemmed_value)
                .unwrap_or(value)
                .to_string();
            stemmed
        }
    }

    #[test]
    fn test_should_inject() {
        let path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine_music");

        let tdir = tempdir().unwrap();
        dir::copy(path, tdir.as_ref(), &dir::CopyOptions::new()).unwrap();
        let engine_dir = tdir.as_ref().join("nlu_engine_music");
        let engine_shared_resources = load_engine_shared_resources(&engine_dir).unwrap();

        let stems = vec![("funky", "funk")].into_iter().collect();
        let stemmer = MockedStemmer { values: stems };
        let mocked_resources = Arc::new(SharedResources {
            builtin_entity_parser: engine_shared_resources
                .as_ref()
                .builtin_entity_parser
                .clone(),
            custom_entity_parser: engine_shared_resources
                .as_ref()
                .custom_entity_parser
                .clone(),
            gazetteers: engine_shared_resources.as_ref().gazetteers.clone(),
            stemmer: Some(Arc::new(stemmer.clone())),
            word_clusterers: HashMap::new(),
            stop_words: HashSet::new(),
        });

        // Behaviour before injection
        let nlu_engine =
            SnipsNluEngine::from_path_with_resources(&engine_dir, mocked_resources.clone())
                .unwrap();
        let parsing = nlu_engine
            .parse("je souhaiterais écouter l'album thisisthebestalbum", None)
            .unwrap();
        assert_eq!(
            parsing.intent.intent_name,
            Some("adri:PlayMusic".to_string())
        );
        assert_eq!(parsing.slots, vec![]);
        let parsing = nlu_engine
            .parse("je voudrais ecouter ma playlist funk", None)
            .unwrap();
        assert_eq!(
            parsing.intent.intent_name,
            Some("adri:PlayMusic".to_string())
        );
        assert_eq!(parsing.slots, vec![]);

        // values to inject
        let values = vec![
            (
                "snips/musicAlbum".to_string(),
                "Thisisthebestalbum".to_string(),
            ),
            ("playlist".to_string(), "funky".to_string()),
        ];

        let mut injector = NluInjector::new(&engine_dir)
            .from_vanilla(true)
            .shared_resources(mocked_resources);

        for (entity, value) in values {
            injector = injector.add_value(entity, value);
        }

        injector.inject().unwrap();

        let injected_resources = load_engine_shared_resources(&engine_dir).unwrap();

        let mocked_injected_resources = SharedResources {
            builtin_entity_parser: injected_resources.builtin_entity_parser.clone(),
            custom_entity_parser: injected_resources.custom_entity_parser.clone(),
            gazetteers: injected_resources.gazetteers.clone(),
            stemmer: Some(Arc::new(stemmer)),
            word_clusterers: injected_resources.word_clusterers.clone(),
            stop_words: HashSet::new(),
        };

        let nlu_engine = SnipsNluEngine::from_path_with_resources(
            &engine_dir,
            Arc::new(mocked_injected_resources),
        )
        .unwrap();

        // Behavior after injection
        let parsing = nlu_engine
            .parse("je souhaiterais écouter l'album thisisthebestalbum", None)
            .unwrap();
        assert_eq!(
            parsing.intent.intent_name,
            Some("adri:PlayMusic".to_string())
        );
        let ground_true_slots = vec![Slot {
            raw_value: "thisisthebestalbum".to_string(),
            range: 32..50,
            entity: "snips/musicAlbum".to_string(),
            slot_name: "musicAlbum".to_string(),
            value: SlotValue::MusicAlbum(StringValue::from("Thisisthebestalbum")),
            confidence_score: None,
        }];
        assert_eq!(parsing.slots, ground_true_slots);

        let parsing = nlu_engine
            .parse("je voudrais ecouter ma playlist funk", None)
            .unwrap();
        assert_eq!(
            parsing.intent.intent_name,
            Some("adri:PlayMusic".to_string())
        );
        let ground_true_slots = vec![Slot {
            raw_value: "funk".to_string(),
            range: 32..36,
            entity: "playlist".to_string(),
            slot_name: "playlist".to_string(),
            value: SlotValue::Custom(StringValue::from("funky")),
            confidence_score: None,
        }];
        assert_eq!(parsing.slots, ground_true_slots);
    }
}
