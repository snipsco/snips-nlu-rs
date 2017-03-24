use std::io::prelude::*;
use std::collections::HashSet;
use std::fs::File;

use errors::*;
use serde_json::from_reader;

pub trait Gazetteer {
    fn contains(&self, value: &str) -> bool;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl HashSetGazetteer {
    pub fn new(r: &mut Read) -> Result<HashSetGazetteer> {
        let vec: Vec<String> = from_reader(r)?;
        Ok(HashSetGazetteer { values: vec.iter().cloned().collect() }) // TODO: Check if clone is necessary
    }
}

impl Gazetteer for HashSetGazetteer {
    fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
    }
}

#[cfg(test)]
mod tests {
    use super::HashSetGazetteer;
    use FileConfiguration;

    #[test]
    fn gazetteer_work() {
        let path = ::file_path("snips-sdk-gazetteers/gazetteers/action_verbs_infinitive.json");

        assert!(HashSetGazetteer::new(File::open(path)).is_ok())
    }
}
