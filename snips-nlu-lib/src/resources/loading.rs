use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use serde_json;
use snips_nlu_ontology::{GazetteerEntityKind, IntoBuiltinEntityKind, Language};
use snips_nlu_ontology_parsers::{BuiltinEntityParserConfiguration, GazetteerEntityConfiguration};

use builtin_entity_parsing::CachingBuiltinEntityParser;
use errors::*;
use failure::ResultExt;
use resources::SharedResources;
use resources::gazetteer::HashSetGazetteer;
use resources::word_clusterer::HashMapWordClusterer;
use resources::stemmer::HashMapStemmer;

#[derive(Debug, Deserialize, Clone)]
pub struct ResourcesMetadata {
    language: String,
    gazetteers: Option<Vec<String>>,
    word_clusters: Option<Vec<String>>,
    stems: Option<String>,
    gazetteer_entities: Option<Vec<String>>,
}

pub fn load_language_resources<P: AsRef<Path>>(
    resources_dir: P,
) -> Result<Arc<SharedResources>> {
    let metadata_file_path = resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(&metadata_file_path)?;
    let metadata: ResourcesMetadata = serde_json::from_reader(metadata_file)
        .with_context(|_|
            format!("Cannot deserialize resources metadata file '{:?}'", metadata_file_path))?;
    let stemmer = load_stemmer(&resources_dir, &metadata)?;
    let gazetteers = load_gazetteers(&resources_dir, &metadata)?;
    let word_clusterers = load_word_clusterers(&resources_dir, &metadata)?;
    let builtin_entity_parser = load_builtin_entity_parser(resources_dir, metadata)?;

    Ok(Arc::new(SharedResources {
        builtin_entity_parser,
        gazetteers,
        stemmer,
        word_clusterers,
    }))
}

fn load_builtin_entity_parser<P: AsRef<Path>>(
    resources_dir: P,
    metadata: ResourcesMetadata
) -> Result<Arc<CachingBuiltinEntityParser>> {
    let language = Language::from_str(&metadata.language)?;
    let parser_config = if let Some(gazetteer_entities) = metadata.gazetteer_entities {
        let gazetteer_entities_path = resources_dir
            .as_ref()
            .join("gazetteer_entities");
        let entities = gazetteer_entities.iter()
            .map(|label| Ok(GazetteerEntityKind::from_identifier(label)?))
            .collect::<Result<Vec<_>>>()?;
        let gazetteer_entity_configurations = entities.iter()
            .map(|entity| {
                let path = gazetteer_entities_path
                    .join(entity.to_string().to_lowercase());
                GazetteerEntityConfiguration {
                    builtin_entity_name: entity.identifier().to_string(),
                    resource_path: path,
                }
            })
            .collect();
        BuiltinEntityParserConfiguration {
            language,
            gazetteer_entity_configurations,
        }
    } else {
        BuiltinEntityParserConfiguration {
            language,
            gazetteer_entity_configurations: vec![],
        }
    };
    Ok(Arc::new(CachingBuiltinEntityParser::new(parser_config, 1000)?))
}

fn load_stemmer<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<Option<Arc<HashMapStemmer>>> {
    if let Some(stems) = metadata.stems.as_ref() {
        let stemming_directory = resources_dir.as_ref().join("stemming");
        let stems_path = stemming_directory
            .join(stems)
            .with_extension("txt");
        let stems_reader = File::open(&stems_path)
            .with_context(|_| format!("Cannot open stems file {:?}", stems_path))?;
        let stemmer = HashMapStemmer::from_reader(stems_reader)
            .with_context(|_| format!("Cannot read stems file {:?}", stems_path))?;
        Ok(Some(Arc::new(stemmer)))
    } else {
        Ok(None)
    }
}

fn load_gazetteers<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<HashMap<String, Arc<HashSetGazetteer>>> {
    let mut gazetteers: HashMap<String, Arc<HashSetGazetteer>> = HashMap::new();
    if let Some(gazetteer_names) = metadata.gazetteers.as_ref() {
        let gazetteers_directory = resources_dir.as_ref().join("gazetteers");
        for gazetteer_name in gazetteer_names {
            let gazetteer_path = gazetteers_directory
                .join(gazetteer_name.clone())
                .with_extension("txt");
            let file = File::open(&gazetteer_path)
                .with_context(|_| format!("Cannot open gazetteer file {:?}", gazetteer_path))?;
            let gazetteer = HashSetGazetteer::from_reader(file)
                .with_context(|_| format!("Cannot read gazetteer file {:?}", gazetteer_path))?;
            gazetteers.insert(gazetteer_name.to_string(), Arc::new(gazetteer));
        }
    }
    Ok(gazetteers)
}


fn load_word_clusterers<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<HashMap<String, Arc<HashMapWordClusterer>>> {
    let mut word_clusterers: HashMap<String, Arc<HashMapWordClusterer>> = HashMap::new();
    if let Some(word_clusters) = metadata.word_clusters.as_ref() {
        let word_clusters_directory = resources_dir.as_ref().join("word_clusters");
        for clusters_name in word_clusters {
            let clusters_path = word_clusters_directory
                .join(clusters_name.clone())
                .with_extension("txt");
            ;
            let word_clusters_reader = File::open(&clusters_path)
                .with_context(|_| format!("Cannot open word clusters file {:?}", clusters_path))?;
            let word_clusterer = HashMapWordClusterer::from_reader(word_clusters_reader)
                .with_context(|_| format!("Cannot read word clusters file {:?}", clusters_path))?;
            word_clusterers.insert(clusters_name.to_string(), Arc::new(word_clusterer));
        }
    }
    Ok(word_clusterers)
}
