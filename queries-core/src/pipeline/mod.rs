use std::collections::HashMap;
use std::ops::Range;

use errors::*;

pub mod deep;

pub type Probability = f32;

type Prediction = usize;

type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SlotValue {
    value: String,
    range: Range<usize>,
}
type Slots = HashMap<String, Vec<SlotValue>>;

#[derive(Serialize, Debug)]
pub struct IntentClassifierResult {
    pub name: String,
    pub probability: Probability,
}

pub trait IntentParser {
    fn get_intent(&self, input: &str, probability_threshold: f32, entities: &str) -> Result<Vec<IntentClassifierResult>>;

    fn get_entities(&self, input: &str, intent_name: &str, entities: &str) -> Result<Slots>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input : &I) -> O;
}

trait ClassifierWrapper<I, O> {
    fn run(&self, input : &I) -> Result<O>;
}

