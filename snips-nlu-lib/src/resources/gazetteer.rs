use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::iter::FromIterator;
use std::path::Path;
use std::sync::{Arc, Mutex};

use errors::*;
use failure::ResultExt;
use snips_nlu_ontology::Language;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl HashSetGazetteer {
    fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let reader = BufReader::new(reader);
        let mut values = HashSet::<String>::new();
        for line in reader.lines() {
            let word = line?;
            if !word.is_empty() {
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
}

pub fn load_gazetteer<P: AsRef<Path>>(
    name: String,
    language: Language,
    path: P,
) -> Result<()> {
    let configuration = GazetteerConfiguration { name, language };

    if GAZETTEERS.lock().unwrap().contains_key(&configuration) {
        return Ok(());
    }
    let file = File::open(&path)
        .with_context(|_| format!("Cannot open gazetteer file '{:?}'", path.as_ref()))?;
    let gazetteer = HashSetGazetteer::from_reader(file)?;
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
) -> Result<Arc<HashSetGazetteer>> {
    let configuration = GazetteerConfiguration { name, language };
    GAZETTEERS
        .lock()
        .unwrap()
        .get(&configuration)
        .cloned()
        .ok_or_else(||
            format_err!("Cannot find gazetteer with configuration {:?}", configuration))
}

pub fn clear_gazetteers() {
    GAZETTEERS
        .lock()
        .unwrap()
        .clear();
}

#[cfg(test)]
mod tests {
    use super::{Gazetteer, HashSetGazetteer};

    #[test]
    fn hashset_gazetteer_works() {
        // Given
        let gazetteer: &[u8] = r#"
dog
cat
bear
crocodile"#.as_ref();

        // When
        let gazetteer = HashSetGazetteer::from_reader(gazetteer);

        // Then
        assert!(gazetteer.is_ok());
        let gazetteer = gazetteer.unwrap();
        assert!(gazetteer.contains("dog"));
        assert!(gazetteer.contains("crocodile"));
        assert!(!gazetteer.contains("bird"));
    }
}
