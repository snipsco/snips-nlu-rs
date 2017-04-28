use std::collections::{HashSet, HashMap};
use std::io::prelude::*;
#[cfg(test)]
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;

use errors::*;
use csv;
use fst;
#[cfg(test)]
use serde_json;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

#[cfg(test)]
/// Toy version of a gazetteer wrapping a hashmap, that can easily be used in tests
pub struct HashSetGazetteer {
    values: HashSet<String>,
}

#[cfg(test)]
impl HashSetGazetteer {
    pub fn new(r: &mut Read) -> Result<HashSetGazetteer> {
        let vec: Vec<String> = serde_json::from_reader(r)
            .map_err(|err| format!("could not parse json: {:?}", err))
            .unwrap();
        Ok(HashSetGazetteer { values: HashSet::from_iter(vec) })
    }
}

#[cfg(test)]
impl Gazetteer for HashSetGazetteer {
    fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
    }
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
            let key = GazetteerKey {
                lang: lang,
                category: cat,
                name: name,
                version: version
            };
            let id_set = ids.split(",").map(|it|
                it.parse()
                    .map_err(|e|
                        format!("Gazetteer header parsing error for {:?} : {:?}", &key, e))
                    .unwrap())
                .collect::<HashSet<u64>>();
            header.insert(key, Arc::new(id_set));
        }

        Ok(Arc::new(header))
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
    use std::fs::File;
    use file_path;
    use super::{HashSetGazetteer, FstGazetteerFactory, Gazetteer, GazetteerKey};

    #[test]
    fn hashset_gazetteer_works() {
        let data = r#"["abc", "xyz"]"#;
        let gazetteer = HashSetGazetteer::new(&mut data.as_bytes());
        assert!(gazetteer.is_ok());
        let gazetteer = gazetteer.unwrap();
        assert!(gazetteer.contains("abc"));
        assert!(!gazetteer.contains("def"));
    }

    #[test]
    fn fst_gazetteer_factory_works() {
        let mut header_reader = File::open(file_path("tests/gazetteer/header.txt")).unwrap();
        let factory = FstGazetteerFactory::new_mmap(file_path("tests/gazetteer/words.fst"),
                                                    &mut header_reader);
        let factory = factory.unwrap();

        let weekdays = factory.get_gazetteer(&GazetteerKey {
            lang: "en".to_string(),
            category: "date_and_time".to_string(),
            name: "weekdays".to_string(),
            version: "c1a55db201e23372076c6cc77177ed1ad2393f56".to_string()
        }).unwrap();

        assert!(weekdays.contains("thursday"));
        assert!(!weekdays.contains("furzeday"));

        assert!(factory.get_gazetteer(&GazetteerKey {
            lang: "en".to_string(),
            category: "some unexisitng category".to_string(),
            name: "weekdays".to_string(),
            version: "c1a55db201e23372076c6cc77177ed1ad2393f56".to_string()
        }).is_err())
    }
}
