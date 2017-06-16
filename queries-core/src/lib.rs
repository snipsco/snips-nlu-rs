extern crate base64;
extern crate crfsuite;
extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ndarray;
extern crate queries_resources_packed as resources_packed;
extern crate queries_utils as utils;
extern crate regex;
extern crate rustling_ontology;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate yolo;
extern crate zip;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub use builtin_entities::ontology::*;
pub use errors::*;
pub use pipeline::nlu_engine::{SnipsNLUEngine, TaggedEntity};
pub use pipeline::configuration::{NLUEngineConfigurationConvertible,
                                  NLUEngineConfiguration,
                                  BinaryBasedConfiguration,
                                  FileBasedConfiguration};
pub use pipeline::{IntentClassifierResult,
                   IntentParserResult,
                   Slot,
                   SlotValue};
pub use utils::miscellaneous::file_path;

#[cfg(test)]
mod testutils;

pub mod errors;
mod builtin_entities;
mod models;
mod pipeline;
