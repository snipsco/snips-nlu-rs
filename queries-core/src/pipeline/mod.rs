pub mod feature_processor;
pub mod intent_classifier;
pub mod tokens_classifier;
pub mod slot_filler;

pub type Probability = f32;

pub type BoxedClassifier = Box<::models::tf::Classifier + Send + Sync>;
