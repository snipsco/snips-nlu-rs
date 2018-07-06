use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;
use std::path::Path;
use std::sync::{Arc, Mutex};

use errors::*;
use snips_nlu_ontology::Language;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl HashSetGazetteer {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(&file);
        let mut values = HashSet::<String>::new();
        for line in reader.lines() {
            let word = line?;
            if word.len() > 0 {
                values.insert(word);
            }
        }
        Ok(Self { values })
    }
}

impl<I> From<I> for HashSetGazetteer where I: Iterator<Item=String> {
    fn from(values_it: I) -> Self {
        Self {
            values: HashSet::from_iter(values_it),
        }
    }
}

impl Gazetteer for HashSetGazetteer {
    fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
    }
}

lazy_static! {
    static ref GAZETTEERS: Mutex<HashMap<GazetteerConfiguration, Arc<HashSetGazetteer>>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GazetteerConfiguration {
    pub name: String,
    pub language: Language,
    pub use_stemming: bool,
}

pub fn load_gazetteer<P: AsRef<Path>>(
    name: String,
    language: Language,
    use_stemming: bool,
    path: P,
) -> Result<()> {
    let configuration = GazetteerConfiguration { name, language, use_stemming };

    if GAZETTEERS.lock().unwrap().contains_key(&configuration) {
        return Ok(());
    }
    let gazetteer = HashSetGazetteer::from_path(path)?;
    GAZETTEERS
        .lock()
        .unwrap()
        .entry(configuration)
        .or_insert_with(|| Arc::new(gazetteer));
    Ok(())
}

pub fn get_gazetteer(
    name: String,
    language: Language,
    use_stemming: bool,
) -> Result<Arc<HashSetGazetteer>> {
    let configuration = GazetteerConfiguration { name, language, use_stemming };
    GAZETTEERS
        .lock()
        .unwrap()
        .get(&configuration)
        .map(|gazetteer| gazetteer.clone())
        .ok_or(format_err!("Cannot find gazetteer with configuration {:?}", configuration))
}

#[cfg(test)]
mod tests {
    use super::{Gazetteer, HashSetGazetteer};
    use utils::file_path;

    #[test]
    fn hashset_gazetteer_works() {
        // Given
        let path = file_path("tests")
            .join("gazetteers")
            .join("animals.txt");

        // When
        let gazetteer = HashSetGazetteer::from_path(path);

        // Then
        assert!(gazetteer.is_ok());
        let gazetteer = gazetteer.unwrap();
        assert!(gazetteer.contains("dog"));
        assert!(gazetteer.contains("crocodile"));
        assert!(!gazetteer.contains("bird"));
    }
}
