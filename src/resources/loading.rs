use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use failure::ResultExt;
use log::info;
use serde_derive::Deserialize;
use snips_nlu_ontology::Language;

use crate::entity_parser::{CachingBuiltinEntityParser, CachingCustomEntityParser};
use crate::errors::*;
use crate::models::nlu_engine::NluEngineModel;
use crate::resources::gazetteer::{Gazetteer, HashSetGazetteer};
use crate::resources::stemmer::{HashMapStemmer, Stemmer};
use crate::resources::word_clusterer::{HashMapWordClusterer, WordClusterer};
use crate::resources::SharedResources;

#[derive(Debug, Deserialize, Clone)]
struct ResourcesMetadata {
    language: String,
    gazetteers: Option<Vec<String>>,
    word_clusters: Option<Vec<String>>,
    stems: Option<String>,
    stop_words: Option<String>,
}

pub fn load_shared_resources<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    resources_dir: P,
    builtin_entity_parser_path: Q,
    custom_entity_parser_path: R,
) -> Result<Arc<SharedResources>> {
    let metadata_file_path = resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(&metadata_file_path)?;
    let metadata: ResourcesMetadata =
        serde_json::from_reader(metadata_file).with_context(|_| {
            format!(
                "Cannot deserialize resources metadata file '{:?}'",
                metadata_file_path
            )
        })?;
    let stemmer = load_stemmer(&resources_dir, &metadata)?;
    let gazetteers = load_gazetteers(&resources_dir, &metadata)?;
    let word_clusterers = load_word_clusterers(&resources_dir, &metadata)?;
    let stop_words = load_stop_words(&resources_dir, &metadata)?;
    let builtin_entity_parser =
        CachingBuiltinEntityParser::from_path(builtin_entity_parser_path, 1000)?;
    let custom_entity_parser =
        CachingCustomEntityParser::from_path(custom_entity_parser_path, 1000)?;

    Ok(Arc::new(SharedResources {
        builtin_entity_parser: Arc::new(builtin_entity_parser),
        custom_entity_parser: Arc::new(custom_entity_parser),
        gazetteers,
        stemmer,
        word_clusterers,
        stop_words,
    }))
}

pub fn load_engine_shared_resources<P: AsRef<Path>>(engine_dir: P) -> Result<Arc<SharedResources>> {
    let nlu_engine_file = engine_dir.as_ref().join("nlu_engine.json");
    let model_file = File::open(&nlu_engine_file)
        .with_context(|_| format!("Could not open nlu engine file {:?}", nlu_engine_file))?;
    let model: NluEngineModel = serde_json::from_reader(model_file)
        .with_context(|_| "Could not deserialize nlu engine json file")?;
    let language = Language::from_str(&model.dataset_metadata.language_code)?;
    let resources_path = engine_dir
        .as_ref()
        .join("resources")
        .join(language.to_string());
    let builtin_parser_path = engine_dir.as_ref().join(&model.builtin_entity_parser);
    let custom_parser_path = engine_dir.as_ref().join(&model.custom_entity_parser);
    load_shared_resources(&resources_path, builtin_parser_path, custom_parser_path)
}

fn load_stemmer<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<Option<Arc<Stemmer>>> {
    if let Some(stems) = metadata.stems.as_ref() {
        let stemming_directory = resources_dir.as_ref().join("stemming");
        let stems_path = stemming_directory.join(stems).with_extension("txt");
        info!("Loading stemmer ({:?}) ...", stems_path);
        let stems_reader = File::open(&stems_path)
            .with_context(|_| format!("Cannot open stems file {:?}", stems_path))?;
        let stemmer = HashMapStemmer::from_reader(stems_reader)
            .with_context(|_| format!("Cannot read stems file {:?}", stems_path))?;
        info!("Stemmer loaded");
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
            info!("Loading gazetteer '{}' ({:?}) ...", gazetteer_name, gazetteer_path);
            let file = File::open(&gazetteer_path)
                .with_context(|_| format!("Cannot open gazetteer file {:?}", gazetteer_path))?;
            let gazetteer = HashSetGazetteer::from_reader(file)
                .with_context(|_| format!("Cannot read gazetteer file {:?}", gazetteer_path))?;
            gazetteers.insert(gazetteer_name.to_string(), Arc::new(gazetteer));
            info!("Gazetteer '{}' loaded", gazetteer_name);
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
            info!("Loading word clusters '{}' ({:?}) ...", clusters_name, clusters_path);
            let word_clusters_reader = File::open(&clusters_path)
                .with_context(|_| format!("Cannot open word clusters file {:?}", clusters_path))?;
            let word_clusterer = HashMapWordClusterer::from_reader(word_clusters_reader)
                .with_context(|_| format!("Cannot read word clusters file {:?}", clusters_path))?;
            word_clusterers.insert(clusters_name.to_string(), Arc::new(word_clusterer));
            info!("Word clusters '{}' loaded", clusters_name);
        }
    }
    Ok(word_clusterers)
}

fn load_stop_words<P: AsRef<Path>>(
    resources_dir: &P,
    metadata: &ResourcesMetadata,
) -> Result<HashSet<String>> {
    if let Some(stop_words_name) = metadata.stop_words.as_ref() {
        let stop_words_path = resources_dir
            .as_ref()
            .join(stop_words_name)
            .with_extension("txt");
        info!("Loading stop words ({:?}) ...", stop_words_path);
        let file = File::open(&stop_words_path)
            .with_context(|_| format!("Cannot open word stop words file {:?}", stop_words_path))?;
        let reader = BufReader::new(file);
        let mut stop_words = HashSet::<String>::new();
        for line in reader.lines() {
            let stop_word = line?;
            if !stop_word.is_empty() {
                stop_words.insert(stop_word);
            }
        }
        info!("Stop words loaded");
        Ok(stop_words)
    } else {
        Ok(HashSet::new())
    }
}
