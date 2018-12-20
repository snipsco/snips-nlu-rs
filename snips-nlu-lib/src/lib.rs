extern crate base64;
extern crate crfsuite;
extern crate csv;
#[cfg(test)]
extern crate dinghy_test;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate lru_cache;
#[macro_use]
extern crate ndarray;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate snips_nlu_ontology;
extern crate snips_nlu_ontology_parsers;
extern crate snips_nlu_utils as nlu_utils;
extern crate tempfile;
extern crate zip;

#[cfg(test)]
#[macro_use]
extern crate maplit;
extern crate snips_nlu_utils;

mod entity_parser;
pub mod models;
pub mod errors;
pub mod injection;
mod intent_classifier;
mod intent_parser;
mod language;
mod nlu_engine;
mod resources;
mod slot_filler;
mod slot_utils;
#[cfg(test)]
mod testutils;
mod utils;

pub const MODEL_VERSION: &str = "0.18.0";

pub use crate::models::*;
pub use crate::errors::*;
pub use crate::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
pub use crate::intent_parser::{DeterministicIntentParser, IntentParser, ProbabilisticIntentParser};
pub use crate::nlu_engine::SnipsNluEngine;
pub use crate::slot_filler::{CRFSlotFiller, SlotFiller};
pub use crate::resources::loading::load_shared_resources;
pub use crate::resources::SharedResources;

pub use snips_nlu_ontology::Language;
pub use nlu_utils::token::{compute_all_ngrams, tokenize_light};
