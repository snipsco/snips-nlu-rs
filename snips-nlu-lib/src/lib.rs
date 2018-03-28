extern crate base64;
extern crate crfsuite;
extern crate csv;
extern crate dinghy_test;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate ndarray;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate snips_nlu_ontology;
extern crate snips_nlu_ontology_parsers;
extern crate snips_nlu_resources_packed as resources_packed;
extern crate snips_nlu_utils as nlu_utils;
extern crate yolo;
extern crate zip;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod errors;
mod configurations;
mod intent_classifier;
mod intent_parser;
mod resources;
mod nlu_engine;
mod slot_utils;
mod slot_filler;
mod utils;
mod language;
#[cfg(test)]
mod testutils;

pub const MODEL_VERSION: &str = "0.13.0";

pub use errors::*;
pub use nlu_engine::SnipsNluEngine;
pub use configurations::{FileBasedConfiguration, NluEngineConfiguration,
                         NluEngineConfigurationConvertible, ZipBasedConfiguration};
pub use nlu_utils::token::{compute_all_ngrams, tokenize_light};
pub use utils::file_path; // This is used by benches
