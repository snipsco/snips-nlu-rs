extern crate queries_preprocessor as preprocessing;
extern crate queries_utils as utils;

extern crate crfsuite;

extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate fst;
extern crate itertools;
extern crate ndarray;
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
pub use pipeline::rule_based::RuleBasedIntentParser;
pub use pipeline::combined::SnipsIntentParser;
pub use pipeline::IntentClassifierResult;
pub use pipeline::IntentParser;
pub use pipeline::IntentParserResult;
pub use pipeline::SlotValue;
pub use utils::file_path;

#[cfg(test)]
mod testutils;

pub mod errors;
mod models;
mod pipeline;
