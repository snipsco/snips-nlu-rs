mod feature_processor;
pub mod tf_classifier_wrapper;
mod intent_configuration;
pub mod intent_parser;
mod slot_filler;

pub use self::intent_parser::DeepIntentParser as IntentParser;
