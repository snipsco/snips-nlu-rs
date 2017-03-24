use std::io::Read;
use std::fs::File;
use std::fs;
use std::path;
use std::collections::HashMap;
use csv;

use errors::*;

use models::gazetteer::{Gazetteer, HashSetGazetteer};
use protobuf;


pub trait AssistantConfig {
    fn get_available_intents_names(&self) -> Result<Vec<String>>;
    fn get_intent_configuration(&self, name: &str) -> Result<Box<IntentConfig>>;
}

pub trait IntentConfig  {
    fn get_file(&self, file_name: &path::Path) -> Result<Box<Read>>;
    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>>;
    fn get_pb_config(&self) -> Result<::protos::intent_configuration::IntentConfiguration> {
        Ok(protobuf::parse_from_reader::<::protos::intent_configuration::IntentConfiguration>(&mut Self::get_file(&self, path::Path::new("config.pb"))?)?)
    }
}

pub struct FileBasedAssistantConfig {
    intents_dir: ::path::PathBuf,
    gazetteers_dir: ::path::PathBuf,

}

impl FileBasedAssistantConfig {
    pub fn new<P: AsRef<path::Path>>(root_dir: P) -> FileBasedAssistantConfig {
        let root_dir = path::PathBuf::from(root_dir.as_ref());
        FileBasedAssistantConfig {
            intents_dir: root_dir.join("intents"),
            gazetteers_dir: root_dir.join("gazetteers"),
        }
    }
}

impl AssistantConfig for FileBasedAssistantConfig {
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

    fn get_intent_configuration(&self, name: &str) -> Result<Box<IntentConfig>> {
        Ok(Box::new(FileBasedIntentConfig::new(self.intents_dir.join(name),
                                   self.gazetteers_dir.clone())?))
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

impl IntentConfig for FileBasedIntentConfig {
    fn get_file(&self, file_name: &path::Path) -> Result<Box<Read>> {
        Ok(Box::new(File::open(&self.intent_dir.join(file_name))?))
    }

    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>> {
        let mapping = &self.gazetteer_mapping[name];
        let path = &self.gazetteer_dir
            .join(&mapping.0).join(format!("{}_{}.json", &name, &mapping.1));
        Ok(Box::new(HashSetGazetteer::new(&mut File::open(path).map_err(|_| format!("Could not load Gazetteer {:?}", path))?)?))
    }
}

