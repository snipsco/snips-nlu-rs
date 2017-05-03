use std::collections::HashMap;
use std::ops::Range;

use errors::*;

pub mod deep;
pub mod light;

pub type Probability = f32;

type Prediction = usize;

type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SlotValue {
    value: String,
    range: Range<usize>,
    entity: Option<String>,
}

type Slots = HashMap<String, Vec<SlotValue>>;

#[derive(Serialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

pub trait IntentParser {
    fn get_intent(&self, input: &str, probability_threshold: f32) -> Result<Vec<IntentClassifierResult>>;
    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input : &I) -> O;
}

pub trait ClassifierWrapper<I, O> {
    fn run(&self, input : &I) -> Result<O>;
}

