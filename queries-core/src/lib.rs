#[macro_use]
extern crate error_chain;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use(stack)]
extern crate ndarray;
extern crate protobuf;
extern crate rayon;
extern crate regex;
extern crate rulinalg;
extern crate unicode_normalization;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tensorflow;

use std::cmp::Ordering;
use std::fs;
use std::path;
use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use models::IntentConfiguration;
use pipeline::Probability;
use pipeline::intent_classifier::IntentClassifier;
use pipeline::tokens_classifier::TokensClassifier;
use pipeline::slot_filler::compute_slots;

pub use preprocessing::preprocess;
pub use errors::*;

#[cfg(test)]
mod testutils;

pub mod errors;
pub mod features;
pub mod models;
pub mod pipeline;
pub mod preprocessing;

#[derive(Serialize, Debug)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

#[derive(Debug, Clone)]
pub struct FileConfiguration {
    root_dir: ::path::PathBuf,
    configuration_dir: ::path::PathBuf,
    intent_classifier_dir: ::path::PathBuf,
    tokens_classifier_dir: ::path::PathBuf,
    gazetteer_dir: ::path::PathBuf,
}

impl FileConfiguration {
    pub fn new<P: AsRef<path::Path>>(root_dir: P) -> FileConfiguration {
        let root_dir = ::path::PathBuf::from(root_dir.as_ref());

        FileConfiguration {
            configuration_dir: root_dir.join("snips-sdk-models-protobuf/configurations"),
            intent_classifier_dir: root_dir.join("snips-sdk-models-protobuf/models/intent_classification"),
            tokens_classifier_dir: root_dir.join("snips-sdk-models-protobuf/models/tokens_classification"),
            gazetteer_dir: root_dir.join("snips-sdk-gazetteers/gazetteers"),
            root_dir: root_dir,
        }
    }

    pub fn default() -> FileConfiguration {
        FileConfiguration::new(file_path("."))
    }

    pub fn configuration_path(&self, classifier_name: &str) -> ::path::PathBuf {
        self.configuration_dir.join(classifier_name).with_extension("pb")
    }

    pub fn intent_classifier_path(&self, classifier_name: &str) -> ::path::PathBuf {
        self.intent_classifier_dir.join(classifier_name).with_extension("pb")
    }

    pub fn tokens_classifier_path(&self, classifier_name: &str) -> ::path::PathBuf {
        self.tokens_classifier_dir.join(classifier_name).with_extension("pb")
    }

    pub fn gazetteer_path(&self, gazetteer_name: &str) -> ::path::PathBuf {
        self.gazetteer_dir.join(gazetteer_name).with_extension("json")
    }

    pub fn available_intents(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.configuration_dir)?;

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
}

pub fn file_path(file_name: &str) -> ::path::PathBuf {
    if ::std::env::var("DINGHY").is_ok() {
        ::std::env::current_exe().unwrap().parent().unwrap().join("test_data/data").join(file_name)
    } else {
        ::path::PathBuf::from("../data").join(file_name)
    }
}

pub struct IntentParser {
    classifiers: HashMap<String, IntentConfiguration>
}

impl IntentParser {
    pub fn new(file_configuration: &FileConfiguration, configurations: Option<&[&str]>) -> Result<IntentParser> {
        let mut classifiers = HashMap::new();

        let configurations_to_load = if let Some(required_configurations) = configurations {
            required_configurations.iter().map(|s| s.to_string()).collect_vec()
        } else {
            file_configuration.available_intents()?
        };

        for ref c in configurations_to_load {
            let intent = IntentConfiguration::new(file_configuration, c)?;
            classifiers.insert(intent.intent_name.to_string(), intent);
        }

        Ok(IntentParser { classifiers: classifiers })
    }

    pub fn run_intent_classifiers(&self, input: &str, probability_threshold: f64, intent_filter: Option<&[&str]>) -> Vec<IntentClassifierResult> {
        assert!(probability_threshold >= 0.0 && probability_threshold <= 1.0, "it's a developer error to pass a probability_threshold between 0.0 and 1.0");

        let preprocessor_result = preprocess(input);

        let mut probabilities: Vec<IntentClassifierResult> = self.classifiers
            .par_iter()
            .filter(|&(name, _)| intent_filter.map(|f| f.contains(&&**name)).unwrap_or(true))
            .map(|(name, intent_configuration)| {
                let probability = intent_configuration.intent_classifier.run(&preprocessor_result);
                IntentClassifierResult { name: name.to_string(), probability: probability }
            })
            .filter(|result| result.probability >= probability_threshold)
            .collect();

        probabilities.sort_by(|a, b| {
            a.probability.partial_cmp(&b.probability).unwrap_or(Ordering::Equal).reverse()
        });

        probabilities
    }

    pub fn run_tokens_classifier(&self, input: &str, intent_name: &str) -> Result<HashMap<String, String>> {
        let preprocessor_result = preprocess(input);

        let intent_configuration = self.classifiers.get(intent_name).ok_or("intent not found")?; // TODO: Should be my own error set ?
        let probabilities = intent_configuration.tokens_classifier.run(&preprocessor_result)?;

        let token_values = preprocessor_result.tokens.iter().map(|token| &*token.value).collect_vec();
        let slot_values = &compute_slots(&*token_values, &probabilities)[1..];
        let slot_names = &intent_configuration.slot_names[1..];

        let mut result = HashMap::new();
        for (name, value) in slot_names.iter().zip(slot_values.iter()) {
            result.insert(name.clone(), value.clone());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use FileConfiguration;

    #[test]
    #[ignore]
    fn list_configurations() {
        let file_configuration = FileConfiguration::default();

        let available_intents = file_configuration.available_intents().unwrap();
        println!("available_intents: {:?}", available_intents);
    }
}
