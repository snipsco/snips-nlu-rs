use std::io::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use serde_json::from_str;

pub trait Gazetteer: Sized {
    fn contains(&self, value: &str) -> bool;
    fn new(json_filename: &str) -> Option<Self>;
}

pub struct HashSetGazetteer {
    values: HashSet<String>,
}

impl Gazetteer for HashSetGazetteer {
    // TODO: To be improved
    fn new(json_filename: &str) -> Option<HashSetGazetteer> {
        let mut f = File::open(gazetteer_file_path(json_filename)).unwrap();
        let mut s = String::new();
        assert!(f.read_to_string(&mut s).is_ok());
        let vec: Vec<String> = from_str(&s).unwrap();
        Some(HashSetGazetteer { values: vec.iter().cloned().collect() })
    }

    fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
    }
}


#[cfg(not(target_os = "android"))]
pub fn gazetteer_file_path(gazetteer_name: &str) -> String {
    format!("../data/snips-sdk-gazetteers/gazetteers/{}.json",
            gazetteer_name)
}

#[cfg(target_os = "android")]
pub fn gazetteer_file_path(gazetteer_name: &str) -> String {
    //TODO find a way to do that better
    format!("/data/local/tmp/snips-queries-data/snips-sdk-gazetteers/gazetteers/{}.json",
            gazetteer_name)
}


#[cfg(test)]
mod tests {
    use super::Gazetteer;
    use super::HashSetGazetteer;

    #[test]
    fn gazetteer_work() {
        assert!(HashSetGazetteer::new("action_verbs_infinitive").is_some())
    }
}
