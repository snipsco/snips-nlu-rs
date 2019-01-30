mod entity_parser;
pub mod errors;
pub mod injection;
mod intent_classifier;
mod intent_parser;
mod language;
pub mod models;
mod nlu_engine;
mod resources;
mod slot_filler;
mod slot_utils;
#[cfg(test)]
mod testutils;
mod utils;

pub const MODEL_VERSION: &str = "0.19.0";

pub use crate::errors::*;
pub use crate::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
pub use crate::intent_parser::{
    DeterministicIntentParser, IntentParser, ProbabilisticIntentParser,
};
pub use crate::models::*;
pub use crate::nlu_engine::SnipsNluEngine;
pub use crate::resources::loading::load_shared_resources;
pub use crate::resources::SharedResources;
pub use crate::slot_filler::{CRFSlotFiller, SlotFiller};

pub use snips_nlu_ontology::Language;
pub use snips_nlu_utils::token::{compute_all_ngrams, tokenize_light};
