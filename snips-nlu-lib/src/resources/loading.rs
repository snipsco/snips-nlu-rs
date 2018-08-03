use std::path::Path;
use std::str::FromStr;

use builtin_entity_parsing::CachingBuiltinEntityParser;
use errors::*;
use failure::ResultExt;
use resources::gazetteer::{clear_gazetteers, load_gazetteer};
use resources::stemmer::{clear_stemmers, load_stemmer};
use resources::word_clusterer::{clear_word_clusterers, load_word_clusterer};
use snips_nlu_ontology::{GazetteerEntityKind, Language};
use snips_nlu_ontology_parsers::{BuiltinEntityParserConfiguration, GazetteerEntityConfiguration};
use serde_json;
use std::fs::File;

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
) -> Result<()> {
    let metadata_file_path = resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(&metadata_file_path)?;
    let metadata: ResourcesMetadata = serde_json::from_reader(metadata_file)
        .with_context(|_|
            format!("Cannot deserialize resources metadata file '{:?}'", metadata_file_path))?;
    let language = Language::from_str(&metadata.language)?;
    if let Some(gazetteer_names) = metadata.gazetteers {
        let gazetteers_directory = resources_dir.as_ref().join("gazetteers");
        for gazetteer_name in gazetteer_names {
            let gazetteer_path = gazetteers_directory
                .join(gazetteer_name.clone())
                .with_extension("txt");
            load_gazetteer(gazetteer_name, language, gazetteer_path)?;
        }
    }

    if let Some(word_clusters) = metadata.word_clusters {
        let word_clusters_directory = resources_dir.as_ref().join("word_clusters");
        for clusters_name in word_clusters {
            let clusters_path = word_clusters_directory
                .join(clusters_name.clone())
                .with_extension("txt");
            ;
            load_word_clusterer(clusters_name, language, clusters_path)?;
        }
    }

    if let Some(stems) = metadata.stems {
        let stemming_directory = resources_dir.as_ref().join("stemming");
        let stems_path = stemming_directory
            .join(stems)
            .with_extension("txt");
        load_stemmer(language, stems_path)?;
    }

    Ok(())
}

pub fn get_builtin_entity_parser<P: AsRef<Path>>(
    resources_dir: P
) -> Result<CachingBuiltinEntityParser> {
    let metadata_file_path = resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(&metadata_file_path)?;
    let metadata: ResourcesMetadata = serde_json::from_reader(metadata_file)
        .with_context(|_|
            format!("Cannot deserialize resources metadata file '{:?}'", metadata_file_path))?;
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
                    parser_threshold: 0.6,
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
            gazetteer_entity_configurations: vec![]
        }
    };
    CachingBuiltinEntityParser::new(parser_config, 1000)
}

pub fn clear_resources() {
    clear_gazetteers();
    clear_stemmers();
    clear_word_clusterers();
}
