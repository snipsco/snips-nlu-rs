use std::io::Read;
use std::fs::File;
use std::fs;
use std::path;
use std::collections::HashMap;
use csv;

use errors::*;

use models::gazetteer::{Gazetteer, HashSetGazetteer};


pub trait AssistantConfig<IC, R, G> where IC: IntentConfig<R, G>, R: Read, G: Gazetteer {
    fn get_available_intents_names(&self) -> Result<Vec<String>>;
    fn get_intent_configuration(&self, name: &str) -> Result<IC>;
}

pub trait IntentConfig<R, G> where R: Read, G: Gazetteer {
    fn get_file<P: AsRef<path::Path>>(&self, file_name: P) -> Result<R>;
    fn get_gazetteer(&self, name: &str) -> Result<G>;
}

pub struct FileBasedAssistantConfig {
    intents_dir: ::path::PathBuf,
    gazetteers_dir: ::path::PathBuf,

}

impl FileBasedAssistantConfig {
    fn new(root_dir: ::path::PathBuf) -> FileBasedAssistantConfig {
        FileBasedAssistantConfig {
            intents_dir:root_dir.join("Intents"),
            gazetteers_dir:root_dir.join("Gazetteers"),
        }
    }
}

impl AssistantConfig<FileBasedIntentConfig, File, HashSetGazetteer> for FileBasedAssistantConfig {
    fn get_available_intents_names(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.intents_dir)?;

        let mut available_intents = vec![];

        // TODO: kill those unwrap
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let stem = path.file_stem().unwrap();
            let result = stem.to_str().unwrap();
            available_intents.push(result.to_string());
        }

        Ok(available_intents)
    }

    fn get_intent_configuration(&self, name: &str) -> Result<FileBasedIntentConfig> {
        FileBasedIntentConfig::new(self.intents_dir.join(name),
                                   self.gazetteers_dir.clone())
    }
}

pub struct FileBasedIntentConfig {
    intent_dir: ::path::PathBuf,
    gazetteer_dir: ::path::PathBuf,
    gazetteer_mapping: HashMap<String, (String, String)> /* name -> (lang, version)*/,
}


impl FileBasedIntentConfig {
    fn new(intent_dir: ::path::PathBuf, gazetteer_dir: ::path::PathBuf) -> Result<FileBasedIntentConfig> {
        let mut csv_reader = csv::Reader::from_file(intent_dir.join("gazetteers.csv"))?;
        let mut mappings = HashMap::new();

        for row in csv_reader.records() {
            let row = row?;
            mappings.insert(row[0].clone(), (row[1].clone(), row[2].clone()));

        }

        Ok(FileBasedIntentConfig {
            intent_dir: intent_dir,
            gazetteer_dir: gazetteer_dir,
            gazetteer_mapping: mappings,
        })
    }
}

impl IntentConfig<File, HashSetGazetteer> for FileBasedIntentConfig {
    fn get_file<P: AsRef<path::Path>>(&self, file_name: P) -> Result<File> {
        Ok(File::open(&self.intent_dir.join(file_name))?)
    }

    fn get_gazetteer(&self, name: &str) -> Result<HashSetGazetteer> {
        let mapping = &self.gazetteer_mapping[name];
        HashSetGazetteer::from(&mut File::open(&self.gazetteer_dir
            .join(&mapping.0).join(format!("{}_{}.pb", &name, &mapping.1)))?)
    }
}


