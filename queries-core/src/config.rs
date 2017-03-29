use std::fs;
use std::path;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use std::sync::Arc;

use csv;
use errors::*;
use models::gazetteer::{Gazetteer, HashSetGazetteer};
use protobuf;
use protos::intent_configuration::IntentConfiguration;

pub trait AssistantConfig {
    fn get_available_intents_names(&self) -> Result<Vec<String>>;
    fn get_intent_configuration(&self, name: &str) -> Result<ArcBoxedIntentConfig>;
}

pub trait IntentConfig {
    fn get_file(&self, file_name: &path::Path) -> Result<Box<Read>>;
    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>>;
    fn get_pb_config(&self) -> Result<IntentConfiguration> {
        let reader = &mut Self::get_file(&self, path::Path::new("config.pb"))?;
        Ok(protobuf::parse_from_reader::<IntentConfiguration>(reader)?)
    }
}

pub type ArcBoxedIntentConfig = Arc<Box<IntentConfig + Send + Sync>>;

pub struct FileBasedAssistantConfig {
    intents_dir: path::PathBuf,
    gazetteers_dir: path::PathBuf,
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

#[cfg(test)]
impl FileBasedAssistantConfig {
    pub fn default() -> FileBasedAssistantConfig {
        FileBasedAssistantConfig::new("../data")
    }
}

impl AssistantConfig for FileBasedAssistantConfig {
    fn get_available_intents_names(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.intents_dir)?;

        let mut available_intents = vec![];

        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                let intent_name = path.file_name()
                    .and_then(|it| it.to_str())
                    .ok_or(format!("invalid unicode in '{:?}'", path))?;
                available_intents.push(intent_name.to_string())
            }
        }

        Ok(available_intents)
    }

    fn get_intent_configuration(&self, name: &str) -> Result<ArcBoxedIntentConfig> {
        Ok(Arc::new(Box::new(FileBasedIntentConfig::new(self.intents_dir.join(name),
                                                        self.gazetteers_dir.clone())?)))
    }
}

pub struct FileBasedIntentConfig {
    intent_dir: path::PathBuf,
    gazetteer_dir: path::PathBuf,
    /* name -> (lang, category, version)*/
    gazetteer_mapping: HashMap<String, (String, String, String)>,
}

impl FileBasedIntentConfig {
    fn new(intent_dir: path::PathBuf,
           gazetteer_dir: path::PathBuf)
           -> Result<FileBasedIntentConfig> {
        let gazetteers_file = &intent_dir.join("gazetteers.csv");
        let mut csv_reader = csv::Reader::from_file(gazetteers_file)
            .map_err(|_| format!("Could not open gazetteers file : '{:?}'", gazetteers_file))?
            .has_headers(false);
        let mut mappings = HashMap::new();

        for row in csv_reader.decode() {
            let (lang, category, name, version) = row?;
            mappings.insert(name, (lang, category, version));
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
        let path = &self.intent_dir.join(file_name);
        let file = File::open(path).map_err(|_| format!("Could not open file '{:?}'", path));
        Ok(Box::new(file?))
    }

    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>> {
        if let Some(mapping) = self.gazetteer_mapping.get(name) {
            let (ref lang, ref category, ref version) = *mapping;
            let path = &self.gazetteer_dir
                .join(&lang)
                .join(&category)
                .join(format!("{}_{}.json", &name, &version));
            let mut file = File::open(path)
                .map_err(|_| format!("Could not load Gazetteer from file '{:?}'", path))?;
            Ok(Box::new(HashSetGazetteer::new(&mut file)?))
        } else {
            bail!("could not get gazetteer for name {}", name)
        }
    }
}
