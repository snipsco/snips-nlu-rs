use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use csv;
use errors::*;
use failure::ResultExt;
use snips_nlu_ontology::Language;

pub trait Stemmer: Send + Sync {
    fn stem(&self, value: &str) -> String;
}

pub struct HashMapStemmer {
    values: HashMap<String, String>
}

impl HashMapStemmer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut values = HashMap::<String, String>::new();
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .flexible(true)
            .has_headers(false)
            .from_reader(reader);

        for record in csv_reader.records() {
            let elements = record?;
            let stem = &elements[0];
            for value in elements.iter().skip(1) {
                values.insert(value.to_string(), stem.to_string());
            }
        }
        Ok(Self { values })
    }
}

impl Stemmer for HashMapStemmer {
    fn stem(&self, value: &str) -> String {
        self.values
            .get(value)
            .map(|v| v.to_string())
            .unwrap_or_else(|| value.to_string())
    }
}

lazy_static! {
    static ref STEMMERS: Mutex<HashMap<Language, Arc<HashMapStemmer>>> =
        Mutex::new(HashMap::new());
}

pub fn load_stemmer<P: AsRef<Path>>(
    language: Language,
    stems_path: P,
) -> Result<()> {
    if STEMMERS.lock().unwrap().contains_key(&language) {
        return Ok(());
    }
    let stems_reader = File::open(stems_path.as_ref())
        .with_context(|_|
            format!("Cannot open stems file '{:?}'", stems_path.as_ref()))?;
    let stemmer = HashMapStemmer::from_reader(stems_reader)?;
    STEMMERS
        .lock()
        .unwrap()
        .entry(language)
        .or_insert_with(|| Arc::new(stemmer));
    Ok(())
}

pub fn get_stemmer(language: Language) -> Option<Arc<HashMapStemmer>> {
    STEMMERS
        .lock()
        .unwrap()
        .get(&language)
        .cloned()
}

pub fn clear_stemmers() {
    STEMMERS
        .lock()
        .unwrap()
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_stemmer_works() {
        // Given
        let stems: &[u8] = r#"
investigate,investigated,investigation
do,done,don't,doing,did,does"#.as_ref();

        // When
        let stemmer = HashMapStemmer::from_reader(stems);

        // Then
        assert!(stemmer.is_ok());
        let stemmer = stemmer.unwrap();
        assert_eq!(stemmer.stem("don't"), "do".to_string());
        assert_eq!(stemmer.stem("does"), "do".to_string());
        assert_eq!(stemmer.stem("unknown"), "unknown".to_string());
    }
}
