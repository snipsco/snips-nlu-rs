use std::io::prelude::*;
use std::collections::HashSet;
use std::iter::FromIterator;

use errors::*;
use serde_json;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl HashSetGazetteer {
    pub fn new(r: &mut Read) -> Result<HashSetGazetteer> {
        let vec: Vec<String> = serde_json::from_reader(r)?;
        Ok(HashSetGazetteer { values: HashSet::from_iter(vec) })
    }
}

impl Gazetteer for HashSetGazetteer {
    fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
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
