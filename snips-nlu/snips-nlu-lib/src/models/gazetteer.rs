use std::collections::HashSet;
use std::convert::From;
#[cfg(test)]
use std::io::prelude::Read;
use std::iter::FromIterator;

use errors::*;
#[cfg(test)]
use serde_json;
use resources_packed::gazetteer_hits;
use snips_nlu_ontology::Language;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

pub struct StaticMapGazetteer {
    name: String,
    language: Language,
}

impl StaticMapGazetteer {
    pub fn new(gazetteer_name: &str, language: Language, use_stemming: bool) -> Result<Self> {
        let stemming_suffix = if use_stemming { "_stem" } else { "" };
        let full_gazetteer_name = format!("{}{}", gazetteer_name, stemming_suffix);
        // Hack to check if gazetteer exists
        gazetteer_hits(language, &full_gazetteer_name, "")?;
        Ok(Self {
            name: full_gazetteer_name,
            language: language,
        })
    }
}

impl Gazetteer for StaticMapGazetteer {
    fn contains(&self, value: &str) -> bool {
        // checked during initialization
        gazetteer_hits(self.language, &self.name, value).unwrap()
    }
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

#[cfg(test)]
impl HashSetGazetteer {
    pub fn new(r: &mut Read) -> Result<HashSetGazetteer> {
        let vec: Vec<String> = serde_json::from_reader(r)
            .map_err(|err| format!("could not parse json: {:?}", err))
            .unwrap();
        Ok(HashSetGazetteer {
            values: HashSet::from_iter(vec),
        })
    }
}

impl<I> From<I> for HashSetGazetteer
where
    I: Iterator<Item = String>,
{
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

#[cfg(test)]
mod tests {
    use super::{Gazetteer, HashSetGazetteer};

    #[test]
    fn hashset_gazetteer_works() {
        let data = r#"["abc", "xyz"]"#;
        let gazetteer = HashSetGazetteer::new(&mut data.as_bytes());
        assert!(gazetteer.is_ok());
        let gazetteer = gazetteer.unwrap();
        assert!(gazetteer.contains("abc"));
        assert!(!gazetteer.contains("def"));
    }
}
