use std::collections::HashMap;
use std::ops::Range;

use errors::*;

pub mod combined;
pub mod deep;
pub mod light;
pub mod probabilistic;

pub type Probability = f32;

type Prediction = usize;

type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct IntentParserResult {
    pub input: String,
    pub likelihood: f32,
    pub intent_name: String,
    pub slots: Slots,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct IntentClassifierResult {
    pub intent_name: String,
    pub probability: Probability,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SlotValue {
    value: String,
    range: Range<usize>,
    entity: String,
}

pub type Slots = HashMap<String, Vec<SlotValue>>;

pub trait IntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>>;
    fn get_intent(&self,
                  input: &str,
                  probability_threshold: f32)
                  -> Result<Vec<IntentClassifierResult>>;
    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots>;
}

trait FeatureProcessor<I, O> {
    fn compute_features(&self, input: &I) -> O;
}

pub trait ClassifierWrapper<I, O> {
    fn run(&self, input: &I) -> Result<O>;
}
