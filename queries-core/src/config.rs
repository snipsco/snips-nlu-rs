use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, Cursor};
use std::path;
use std::sync::{Arc, Mutex};

use csv;
use protobuf;
use regex::Regex;
use yolo::Yolo;
use zip;

use errors::*;
use models::gazetteer::{Gazetteer, HashSetGazetteer};
use protos::intent_configuration::IntentConfiguration;

pub trait AssistantConfig {
    fn get_available_intents_names(&self) -> Result<Vec<String>>;
    fn get_intent_configuration(&self, name: &str) -> Result<ArcBoxedIntentConfig>;
}

pub trait IntentConfig: Send + Sync {
    fn get_file(&self, file_name: &path::Path) -> Result<Box<Read>>;
    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>>;
    fn get_pb_config(&self) -> Result<IntentConfiguration> {
        let reader = &mut Self::get_file(&self, path::Path::new("config.pb"))?;
        Ok(protobuf::parse_from_reader::<IntentConfiguration>(reader)?)
    }
}

pub type ArcBoxedIntentConfig = Arc<Box<IntentConfig>>;

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

pub struct BinaryBasedAssistantConfig<R: Read + Seek + Send + 'static> {
    archive: Arc<Mutex<zip::read::ZipArchive<R>>>,
}

impl<R: Read + Seek + Send + 'static> BinaryBasedAssistantConfig<R> {
    pub fn new(reader: R) -> Result<BinaryBasedAssistantConfig<R>> {
        let zip = zip::ZipArchive::new(reader)?;
        Ok(BinaryBasedAssistantConfig { archive: Arc::new(Mutex::new(zip)) })
    }
}

impl<R: Read + Seek + Send + 'static> AssistantConfig for BinaryBasedAssistantConfig<R> {
    fn get_available_intents_names(&self) -> Result<Vec<String>> {
        lazy_static! {
            static ref INTENT_REGEX: Regex = Regex::new(r"intents/(.+?)/config.pb").yolo();
        }
        let mut locked =
            self.archive.lock().map_err(|_| "Can not take lock on ZipFile. Mutex poisoned")?;

        let ref mut archive = *locked;

        let mut available_intents = vec![];

        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            if let Some(captures) = INTENT_REGEX.captures(file.name()) {
                available_intents.push(captures[1].to_string());
            }
        }
        Ok(available_intents)
    }

    fn get_intent_configuration(&self, name: &str) -> Result<ArcBoxedIntentConfig> {
        Ok(Arc::new(Box::new(BinaryBasedIntentConfig::new(self.archive.clone(), name.to_string())?)))
    }
}

pub struct BinaryBasedIntentConfig<R: Read + Seek + Send + 'static> {
    archive: Arc<Mutex<zip::read::ZipArchive<R>>>,
    intent_name: String,
    /* name -> (lang, category, version)*/
    gazetteer_mapping: HashMap<String, (String, String, String)>,
}

impl<R: Read + Seek + Send + 'static> BinaryBasedIntentConfig<R> {
    fn new(archive: Arc<Mutex<zip::read::ZipArchive<R>>>, name: String) -> Result<BinaryBasedIntentConfig<R>> {
        let archive_clone = archive.clone();
        let mut locked = archive_clone.lock()
            .map_err(|_| "Can not take lock on ZipFile. Mutex poisoned")?;

        let ref mut zip_file = *locked;

        let gazetteers_reader = zip_file.by_name(&format!("intents/{}/gazetteers.csv", &name));
        let mut csv_reader = csv::Reader::from_reader(gazetteers_reader?)
            .has_headers(false);
        let mut mappings = HashMap::new();

        for row in csv_reader.decode() {
            let (lang, category, name, version) = row?;
            mappings.insert(name, (lang, category, version));
        }

        Ok(BinaryBasedIntentConfig {
            archive: archive,
            intent_name: name,
            gazetteer_mapping: mappings
        })
    }
}

impl<R: Read + Seek + Send + 'static> IntentConfig for BinaryBasedIntentConfig<R> {
    fn get_file(&self, file_name: &path::Path) -> Result<Box<Read>> {
        let mut locked = self.archive.lock()
            .map_err(|_| "Can not take lock on ZipFile. Mutex poisoned")?;

        let ref mut zip_file = *locked;
        let file = zip_file.by_name(&format!("intents/{}/{}",
                                             self.intent_name,
                                             &file_name.to_str().ok_or("Utf8 error on path name")?));

        let mut result = vec![];
        file?.read_to_end(&mut result)?;

        Ok(Box::new(Cursor::new(result)))
    }

    fn get_gazetteer(&self, name: &str) -> Result<Box<Gazetteer>> {
        if let Some(mapping) = self.gazetteer_mapping.get(name) {
            let (ref lang, ref category, ref version) = *mapping;

            let mut locked = self.archive.lock()
                .map_err(|_| "Can not take lock on ZipFile. Mutex poisoned")?;

            let ref mut zip_file = *locked;

            let file_name = format!("gazetteers/{}/{}/{}_{}.json", lang, category, name, version);
            let file = zip_file.by_name(&file_name);

            Ok(Box::new(HashSetGazetteer::new(&mut file?)?))
        } else {
            bail!("could not get gazetteer for name {}", name)
        }
    }
}


#[cfg(test)]
mod test {
    use std::fs;
    use std::path;
    use super::{BinaryBasedAssistantConfig, AssistantConfig};

    use file_path;

    #[test]
    fn can_decode() {
        let reader = fs::File::open(file_path("tests/zip_files/sample_builtin_config.zip")).unwrap();
        let intent_config = BinaryBasedAssistantConfig::new(reader).unwrap();

        assert!(intent_config.get_available_intents_names().unwrap().len() == 1);
        assert!(intent_config.get_available_intents_names().unwrap()[0] == "BookRestaurant");

        let book_restaurant = intent_config.get_intent_configuration("BookRestaurant").unwrap();

        assert!(book_restaurant.get_gazetteer("meals").unwrap().contains("lunch"));
        assert!(!book_restaurant.get_gazetteer("meals").unwrap().contains("lunch2"));
        let mut test_file = book_restaurant.get_file(path::Path::new("test_file")).unwrap();
        let mut file_content = String::new();

        test_file.read_to_string(&mut file_content).unwrap();

        assert!(file_content.trim() == "Hello, world !")
    }
}
