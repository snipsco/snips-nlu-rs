use std::path::Path;
use std::str::FromStr;

use errors::*;
use resources::gazetteer::load_gazetteer;
use resources::stemmer::load_stemmer;
use resources::word_clusterer::load_word_clusterer;
use snips_nlu_ontology::Language;
use serde_json;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
pub struct ResourcesMetadata {
    language: String,
    gazetteers: Option<Vec<String>>,
    word_clusters: Option<Vec<String>>,
    stems: Option<String>
}

pub fn load_resources<P: AsRef<Path>>(resources_dir: P) -> Result<()> {
    for dir_entry in resources_dir.as_ref().read_dir()? {
        let language_resources_path = dir_entry?.path();
        let metadata_file_path = language_resources_path.join("metadata.json");
        if metadata_file_path.exists() {
            load_language_resources(language_resources_path)?;
        }
    }
    Ok(())
}

pub fn load_language_resources<P: AsRef<Path>>(
    language_resources_dir: P,
) -> Result<()> {
    let metadata_file_path = language_resources_dir.as_ref().join("metadata.json");
    let metadata_file = File::open(metadata_file_path)?;
    let metadata: ResourcesMetadata = serde_json::from_reader(metadata_file)?;
    let language = Language::from_str(&metadata.language)?;
    if let Some(gazetteer_names) = metadata.gazetteers {
        let gazetteers_directory = language_resources_dir.as_ref().join("gazetteers");
        for gazetteer_name in gazetteer_names.into_iter() {
            let gazetteer_path = gazetteers_directory
                .join(gazetteer_name.clone())
                .with_extension("txt");
            let use_stemming = gazetteer_name.ends_with("_stem");
            let name = if use_stemming {
                gazetteer_name[0..(gazetteer_name.len() - 5)].to_string()
            } else {
                gazetteer_name
            };
            load_gazetteer(name, language, use_stemming, gazetteer_path)?;
        }
    }

    if let Some(word_clusters) = metadata.word_clusters {
        let word_clusters_directory = language_resources_dir.as_ref().join("word_clusters");
        for clusters_name in word_clusters.into_iter() {
            let clusters_path = word_clusters_directory
                .join(clusters_name.clone())
                .with_extension("txt");;
            load_word_clusterer(clusters_name, language, clusters_path)?;
        }
    }

    if let Some(stems) = metadata.stems {
        let stemming_directory = language_resources_dir.as_ref().join("stemming");
        let stems_path = stemming_directory
            .join(stems)
            .with_extension("txt");
        load_stemmer(language, stems_path)?;
    }

    Ok(())
}
