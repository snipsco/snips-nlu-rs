use std::collections::HashMap;
use std::ops::Range;

use errors::*;

pub mod deep;
pub mod light;

pub type Probability = f32;

type Prediction = usize;

type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct IntentParserResult {
    pub input: String,
    pub intent_name: String,
    pub slots: Slots,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: Probability,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SlotValue {
    value: String,
    range: Range<usize>,
    entity: String,
}

type Slots = HashMap<String, Vec<SlotValue>>;

pub trait IntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<IntentParserResult>;
    fn get_intent(&self, input: &str, probability_threshold: f32) -> Result<Vec<IntentClassifierResult>>;
    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input : &I) -> O;
}

pub trait ClassifierWrapper<I, O> {
    fn run(&self, input : &I) -> Result<O>;
}

