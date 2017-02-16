use std::io::prelude::*;
use std::collections::HashSet;
use std::fs::File;

use errors::*;
use serde_json::from_str;

use ::FileConfiguration;

pub trait Gazetteer: Sized {
    fn contains(&self, value: &str) -> bool;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl HashSetGazetteer {
    // TODO: To be improved
    pub fn new(file_configuration: &FileConfiguration, gazetteer_name: &str) -> Result<HashSetGazetteer> {
        let gazetteer_path = file_configuration.gazetteer_path(gazetteer_name);

        let mut f = File::open(gazetteer_path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let vec: Vec<String> = from_str(&s)?;
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
       let file_configuration = FileConfiguration::default();
       let gazetteer_name = "action_verbs_infinitive";

       assert!(HashSetGazetteer::new(&file_configuration, gazetteer_name).is_ok())
    }
}
