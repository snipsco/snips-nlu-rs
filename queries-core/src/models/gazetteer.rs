use std::collections::{HashSet, HashMap};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

use errors::*;
use csv;
use fst;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct GazetteerKey {
    pub lang: String,
    pub category: String,
    pub name: String,
    pub version: String,
}

type Header = Arc<HashMap<GazetteerKey, Arc<HashSet<u64>>>>;

#[derive(Clone)]
pub struct FstGazetteerFactory {
    map: Arc<fst::Map>,
    header: Header
}

impl FstGazetteerFactory {
    pub fn new_mmap<F: AsRef<Path>, H: Read>(fst_path: F, header_reader: &mut H) -> Result<Self> {
        Ok(FstGazetteerFactory {
            map: Arc::new(fst::Map::from_path(fst_path)?),
            header: FstGazetteerFactory::build_header(header_reader)?
        })
    }

    pub fn new_ram<R: Read>(fst_bytes: Vec<u8>, header_reader: &mut R) -> Result<Self> {
        Ok(FstGazetteerFactory {
            map: Arc::new(fst::Map::from_bytes(fst_bytes)?),
            header: FstGazetteerFactory::build_header(header_reader)?
        })
    }

    fn build_header<R: Read>(reader: &mut R) -> Result<Header> {
        let mut csv_reader = csv::Reader::from_reader(reader).delimiter(b':').has_headers(false);
        let mut header = HashMap::new();

        for row in csv_reader.decode() {
            let (lang, cat, name, version, ids): (String, String, String, String, String) = row?;
            let id_set = ids.split(",").map(|it| it.parse().unwrap()).collect::<HashSet<u64>>();
            header.insert(
                GazetteerKey {
                    lang: lang,
                    category: cat,
                    name: name,
                    version: version
                },
                Arc::new(id_set));
        }

        return Ok(Arc::new(header))
    }

    pub fn get_gazetteer(&self, key: &GazetteerKey) -> Result<Box<Gazetteer>> {
        if let Some(possible_values) = self.header.get(key) {
            Ok(Box::new(FstGazetteer {
                map: self.map.clone(),
                possible_values: possible_values.clone(),
            }))
        } else {
            bail!("Could not find gazetteer {:?}", key)
        }
    }
}

pub struct FstGazetteer {
    map: Arc<fst::Map>,
    possible_values: Arc<HashSet<u64>>
}

impl Gazetteer for FstGazetteer {
    fn contains(&self, value: &str) -> bool {
        if let Some(v) = self.map.get(value) {
            self.possible_values.contains(&v)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HashSetGazetteer, Gazetteer};

    #[test]
    fn gazetteer_work() {
        let data = r#"["abc", "xyz"]"#;
        let gazetteer = HashSetGazetteer::new(&mut data.as_bytes());
        assert!(gazetteer.is_ok());
        let gazetteer = gazetteer.unwrap();
        assert!(gazetteer.contains("abc"));
        assert!(!gazetteer.contains("def"));
    }
}
