extern crate itertools;

extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

#[macro_use(stack)]
extern crate ndarray;

extern crate unicode_normalization;

extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate protobuf;

extern crate rayon;

extern crate rulinalg;

extern crate tensorflow;

pub mod models;
pub mod preprocessing;
pub mod pipeline;
pub mod features;

pub use preprocessing::preprocess;

#[cfg(test)]
mod testutils;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path;

use itertools::Itertools;
use rayon::prelude::*;

use models::IntentConfiguration;
use pipeline::Probability;
use pipeline::intent_classifier::{IntentClassifier, ProtobufIntentClassifier};
use pipeline::tokens_classifier::{TokensClassifier, ProtobufTokensClassifier};
use pipeline::slot_filler::compute_slots;

pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: Probability,
}

pub struct IntentParser {
    classifiers: HashMap<String, IntentConfiguration>
}

impl IntentParser {
    fn new(configurations: &[&str]) -> IntentParser {
        let mut classifiers = HashMap::new();

        for c in configurations {
            let intent = IntentConfiguration::new(c);
            classifiers.insert(intent.intent_name.to_string(), intent);
        }

        IntentParser {
            classifiers: classifiers,
        }
    }

    pub fn run_intent_classifiers(&self, input: &str, probability_threshold: f64) -> Vec<IntentClassifierResult> {
        assert!(probability_threshold >= 0.0 && probability_threshold <= 1.0, "probability_treshold should be between 0.0 and 1.0");

        let preprocessor_result = preprocess(input);

        let mut probabilities: Vec<IntentClassifierResult> = self.classifiers
        .iter() // FIXME par_iter
        .map(|(name, intent_configuration)| {
            let probability = intent_configuration.intent_classifier.run(&preprocessor_result);
            IntentClassifierResult { intent_name: name.to_string(), probability: probability }
        })
        .filter(|result| result.probability >= probability_threshold)
        .collect();

        probabilities.sort_by(|a, b| {
            a.probability.partial_cmp(&b.probability).unwrap_or(Ordering::Equal).reverse()
        });

        probabilities
    }

    pub fn run_tokens_classifier(&mut self, input: &str, intent_name: &str) -> HashMap<String, String> {
        let preprocessor_result = preprocess(input);

        let intent_configuration = self.classifiers.get_mut(intent_name).unwrap();
        let probabilities = intent_configuration.tokens_classifier.run(&preprocessor_result);

        let token_values = preprocessor_result.tokens.iter().map(|token| &*token.value).collect_vec();
        let slot_values = compute_slots(&*token_values, &probabilities);
        let ref slot_names = intent_configuration.slot_names;

        let mut result = HashMap::new();
        for (name, value) in slot_names.iter().zip(slot_values.iter()) {
            result.insert(name.clone(), name.clone());
        }

        result
    }
}
