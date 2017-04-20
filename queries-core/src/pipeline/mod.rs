use std::collections::HashMap;

use errors::*;

mod intent_configuration;
pub mod deep_feature_processor;
pub mod tf_classifier_wrapper;
pub mod deep_intent_parser;
pub mod slot_filler;

use self::slot_filler::SlotValue;

pub type Probability = f32;

pub type Prediction = usize;

pub type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;

pub trait ClassifierWrapper<I, O> {
    fn run(&self, input : &I) -> Result<O>;
}

pub trait IntentParser {
    fn get_intent(&self, input: &str, probability_threshold: f32, entities: &str) -> Result<Vec<IntentClassifierResult>>;

    fn get_entities(&self, input: &str, intent_name: &str, entities: &str) -> Result<Slots>;
}

type Slots = HashMap<String, Vec<SlotValue>>;

#[derive(Serialize, Debug)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

pub trait FeatureProcessor<I, O> {
    fn compute_features(&self, input : &I) -> O;
}
