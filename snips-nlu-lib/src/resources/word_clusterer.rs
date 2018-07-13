use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use csv;
use errors::*;
use failure::ResultExt;
use snips_nlu_ontology::Language;
use std::fs::File;

pub trait WordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct HashMapWordClusterer {
    values: HashMap<String, String>
}

impl HashMapWordClusterer {
    fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(reader);
        let mut values = HashMap::<String, String>::new();
        for record in csv_reader.records() {
            let elements = record?;
            values.insert(elements[0].to_string(), elements[1].to_string());
        }

        Ok(Self { values })
    }
}

impl WordClusterer for HashMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        self.values
            .get(word)
            .map(|v| v.to_string())
    }
}

lazy_static! {
    static ref WORD_CLUSTERERS: Mutex<HashMap<WordClustererConfiguration, Arc<HashMapWordClusterer>>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordClustererConfiguration {
    language: Language,
    clusters_name: String,
}

pub fn load_word_clusterer<P: AsRef<Path>>(
    clusters_name: String,
    language: Language,
    path: P,
) -> Result<()> {
    let configuration = WordClustererConfiguration { language, clusters_name };
    if WORD_CLUSTERERS.lock().unwrap().contains_key(&configuration) {
        return Ok(());
    }

    let word_clusters_reader = File::open(path.as_ref())
        .with_context(|_| format!("Cannot open word clusters file '{:?}'", path.as_ref()))?;
    let word_clusterer = HashMapWordClusterer::from_reader(word_clusters_reader)?;
    WORD_CLUSTERERS
        .lock()
        .unwrap()
        .entry(configuration)
        .or_insert_with(|| Arc::new(word_clusterer));
    Ok(())
}

pub fn get_word_clusterer(
    clusters_name: String,
    language: Language,
) -> Result<Arc<HashMapWordClusterer>> {
    let configuration = WordClustererConfiguration { clusters_name, language };
    WORD_CLUSTERERS
        .lock()
        .unwrap()
        .get(&configuration)
        .cloned()
        .ok_or_else(||
            format_err!("Cannot find word clusterer with configuration {:?}", configuration))
}

pub fn clear_word_clusterers() {
    WORD_CLUSTERERS
        .lock()
        .unwrap()
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_word_clusterer_works() {
        // Given
        let clusters: &[u8] = r#"
hello	1111111111111
world	1111110111111"#.as_ref();

        // When
        let clusterer = HashMapWordClusterer::from_reader(clusters);

        // Then
        assert!(clusterer.is_ok());
        let clusterer = clusterer.unwrap();
        assert_eq!(clusterer.get_cluster("hello"), Some("1111111111111".to_string()));
        assert_eq!(clusterer.get_cluster("world"), Some("1111110111111".to_string()));
        assert_eq!(clusterer.get_cluster("unknown"), None);
    }
}
