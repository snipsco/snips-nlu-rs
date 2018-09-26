use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use serde_json;

use entity_parser::{CachingBuiltinEntityParser, CachingCustomEntityParser};
use errors::*;
use failure::ResultExt;
use resources::SharedResources;
use resources::gazetteer::{Gazetteer, HashSetGazetteer};
use resources::word_clusterer::{HashMapWordClusterer, WordClusterer};
use resources::stemmer::{HashMapStemmer, Stemmer};

#[derive(Debug, Deserialize, Clone)]
pub struct ResourcesMetadata {
    language: String,
    gazetteers: Option<Vec<String>>,
    word_clusters: Option<Vec<String>>,
    stems: Option<String>,
}

pub fn load_shared_resources<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    resources_dir: P,
    builtin_entity_parser_path: Q,
    custom_entity_parser_path: R,
) -> Result<Arc<SharedResources>> {
    let metadata_file_path = resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(&metadata_file_path)?;
    let metadata: ResourcesMetadata = serde_json::from_reader(metadata_file)
        .with_context(|_|
            format!("Cannot deserialize resources metadata file '{:?}'", metadata_file_path))?;
    let stemmer = load_stemmer(&resources_dir, &metadata)?;
    let gazetteers = load_gazetteers(&resources_dir, &metadata)?;
    let word_clusterers = load_word_clusterers(&resources_dir, &metadata)?;
    let builtin_entity_parser = CachingBuiltinEntityParser::from_path(builtin_entity_parser_path, 1000)?;
    let custom_entity_parser = CachingCustomEntityParser::from_path(custom_entity_parser_path, 1000)?;

    Ok(Arc::new(SharedResources {
        builtin_entity_parser: Arc::new(builtin_entity_parser),
        custom_entity_parser: Arc::new(custom_entity_parser),
        gazetteers,
        stemmer,
        word_clusterers,
    }))
}

fn load_stemmer<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<Option<Arc<Stemmer>>> {
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
) -> Result<HashMap<String, Arc<Gazetteer>>> {
    let mut gazetteers: HashMap<String, Arc<Gazetteer>> = HashMap::new();
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
) -> Result<HashMap<String, Arc<WordClusterer>>> {
    let mut word_clusterers: HashMap<String, Arc<WordClusterer>> = HashMap::new();
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
