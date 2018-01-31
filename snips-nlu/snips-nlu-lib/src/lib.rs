#![recursion_limit = "128"]

extern crate base64;
extern crate crfsuite;
extern crate csv;
#[cfg(test)]
extern crate dinghy_test;
#[macro_use]
extern crate error_chain;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ndarray;
extern crate snips_nlu_resources_packed as resources_packed;
extern crate nlu_utils;
extern crate snips_nlu_ontology;
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

pub mod errors;
mod builtin_entities;
mod models;
mod pipeline;
mod utils;
mod language;
#[cfg(test)]
mod testutils;

pub use builtin_entities::ontology::*;
pub use errors::*;
pub use pipeline::nlu_engine::SnipsNluEngine;
pub use pipeline::configuration::{NluEngineConfigurationConvertible,
                                  NluEngineConfiguration};
pub use pipeline::assistant_config::{FileBasedConfiguration,
                                     ZipBasedConfiguration};

pub use pipeline::nlu_engine::deprecated::*;
pub use pipeline::assistant_config::deprecated::*;
pub use nlu_utils::token::{compute_all_ngrams, tokenize_light};
#[cfg(test)]
pub use utils::file_path;
