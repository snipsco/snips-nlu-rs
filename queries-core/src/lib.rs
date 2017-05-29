extern crate base64;
extern crate crfsuite;
extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate fst;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ndarray;
extern crate queries_preprocessor as preprocessing;
extern crate queries_resources_packed as resources_packed;
extern crate queries_utils as utils;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate yolo;
extern crate zip;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub use errors::*;
pub use models::gazetteer::GazetteerKey;
pub use pipeline::nlu_engine::SnipsNLUEngine;
pub use pipeline::IntentClassifierResult;
pub use pipeline::IntentParserResult;
pub use pipeline::Slot;
pub use utils::file_path;

#[cfg(test)]
mod testutils;

pub mod errors;
mod models;
mod pipeline;
