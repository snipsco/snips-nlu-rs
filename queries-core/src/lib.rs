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
extern crate yolo;
extern crate csv;

use std::cmp::Ordering;
use std::path;
use std::sync;
use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use models::IntentConfiguration;
use pipeline::Probability;
use pipeline::intent_classifier::IntentClassifier;
use pipeline::tokens_classifier::TokensClassifier;
use pipeline::slot_filler::compute_slots;
use yolo::Yolo;

pub use preprocessing::preprocess;
pub use errors::*;

use config::AssistantConfig;

#[cfg(test)]
mod testutils;

pub mod errors;
mod features;
mod models;
mod pipeline;
mod preprocessing;

mod protos;

#[derive(Serialize, Debug)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

pub mod config;

pub fn file_path(file_name: &str) -> ::path::PathBuf {
    if ::std::env::var("DINGHY").is_ok() {
        ::std::env::current_exe().unwrap().parent().unwrap().join("test_data/data").join(file_name)
    } else {
        ::path::PathBuf::from("../data").join(file_name)
    }
}



pub struct IntentParser {
    classifiers: HashMap<String, IntentConfiguration>,
}

impl IntentParser {
    pub fn new(assistant_config: &AssistantConfig) -> Result<IntentParser> {
        let mut classifiers = HashMap::new();

        for ref c in assistant_config.get_available_intents_names()? {
            let intent = IntentConfiguration::new(sync::Arc::new(assistant_config.get_intent_configuration(c)?))?;
            classifiers.insert(intent.intent_name.to_string(), intent);
        }

        Ok(IntentParser { classifiers: classifiers })
    }

    pub fn run_intent_classifiers(&self, input: &str, probability_threshold: f32) -> Vec<IntentClassifierResult> {
        assert!(probability_threshold >= 0.0 && probability_threshold <= 1.0, "it's a developer error to pass a probability_threshold between 0.0 and 1.0");

        let preprocessor_result = preprocess(input);

        let mut probabilities: Vec<IntentClassifierResult> = self.classifiers
            .par_iter()
            .map(|(name, intent_configuration)| {
                let probability = intent_configuration.intent_classifier.run(&preprocessor_result);
                // TODO remove this YOLO
                IntentClassifierResult { name: name.to_string(), probability: probability.yolo() }
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



        let slot_names = &intent_configuration.slot_names;

        println!("{:?} =>  {:?}", slot_names, probabilities);
        let slot_values = &compute_slots(&*token_values, slot_names.len(), &probabilities);

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
