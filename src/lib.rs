#![allow(
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::module_inception
)]

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

pub const MODEL_VERSION: &str = "0.20.0";

pub extern crate snips_nlu_ontology as ontology;
pub use crate::errors::*;
pub use crate::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
pub use crate::intent_parser::{
    DeterministicIntentParser, IntentParser, LookupIntentParser, ProbabilisticIntentParser,
};
pub use crate::models::*;
pub use crate::nlu_engine::SnipsNluEngine;
pub use crate::resources::loading::load_shared_resources;
pub use crate::resources::SharedResources;
pub use crate::slot_filler::{CRFSlotFiller, SlotFiller};
pub use snips_nlu_ontology::Language;
