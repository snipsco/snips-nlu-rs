use errors::*;

mod intent_configuration;
pub mod feature_processor;
pub mod tf_classifier_wrapper;
pub mod intent_parser;
pub mod slot_filler;

pub type Probability = f32;

pub type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;


pub trait ClassifierWrapper<I, O> {
    fn run(&self, input : &I) -> Result<O>;
}
