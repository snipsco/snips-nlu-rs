use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

use errors::*;
use snips_nlu_ontology::Language;

pub trait WordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct HashMapWordClusterer {
    values: HashMap<String, String>
}

impl HashMapWordClusterer {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = File::open(path)?;
        let file = BufReader::new(&f);
        let mut values = HashMap::<String, String>::new();
        for line in file.lines() {
            let l = line?;
            let elements: Vec<&str> = l.split("\t").collect();
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
    let word_clusterer = HashMapWordClusterer::from_path(path)?;
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
        .map(|word_clusterer| word_clusterer.clone())
        .ok_or(format_err!("Cannot find word clusterer with configuration {:?}", configuration))
}
