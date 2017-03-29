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
extern crate unicode_normalization;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tensorflow;
extern crate yolo;
extern crate csv;
extern crate zip;

use std::cmp::Ordering;
use std::path;
use std::collections::HashMap;

use rayon::prelude::*;

use models::IntentConfiguration;
use pipeline::Probability;
use pipeline::intent_classifier::IntentClassifier;
use pipeline::tokens_classifier::TokensClassifier;
use pipeline::slot_filler::compute_slots;

pub use preprocessing::preprocess;
pub use errors::*;
pub use pipeline::slot_filler::Token;

use config::AssistantConfig;

#[cfg(test)]
mod testutils;

pub mod errors;
pub mod config;
mod features;
mod models;
pub mod pipeline;
pub mod preprocessing;
mod protos;

#[derive(Serialize, Debug)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

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
            let intent_config = assistant_config.get_intent_configuration(c)?;
            let intent = IntentConfiguration::new(intent_config)?;
            classifiers.insert(intent.intent_name.to_string(), intent);
        }

        Ok(IntentParser { classifiers: classifiers })
    }

    pub fn run_intent_classifiers(&self,
                                  input: &str,
                                  probability_threshold: f32)
                                  -> Result<Vec<IntentClassifierResult>> {
        if probability_threshold < 0.0 || probability_threshold > 1.0 {
            bail!("it's a developer error to pass a probability_threshold between 0.0 and 1.0")
        }

        let preprocessor_result = preprocess(input);

        let mut probabilities: Vec<IntentClassifierResult> = self.classifiers
            .par_iter()
            .map(|(name, intent_configuration)| {
                let probability = intent_configuration.intent_classifier.run(&preprocessor_result);
                IntentClassifierResult {
                    name: name.to_string(),
                    probability: probability.unwrap_or_else(|e| {
                        println!("could not run intent classifier for {} : {:?}", name, e);
                        -1.0
                    }),
                }
            })
            .filter(|result| result.probability >= probability_threshold)
            .collect();

        probabilities.sort_by(|a, b| {
            a.probability.partial_cmp(&b.probability).unwrap_or(Ordering::Equal).reverse()
        });

        Ok(probabilities)
    }

    pub fn run_tokens_classifier(&self,
                                 input: &str,
                                 intent_name: &str)
                                 -> Result<HashMap<String, Vec<Token>>> {
        let preprocessor_result = preprocess(input);

        let intent_configuration = self.classifiers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;
        let probabilities = intent_configuration.tokens_classifier.run(&preprocessor_result)?;

        let slot_names = &intent_configuration.slot_names;
        let slot_values = &compute_slots(&preprocessor_result, slot_names.len(), &probabilities);

        let mut result = HashMap::new();
        for (name, value) in slot_names.iter().zip(slot_values.iter()) {
            result.insert(name.clone(), value.clone());
        }

        Ok(result)
    }
}
