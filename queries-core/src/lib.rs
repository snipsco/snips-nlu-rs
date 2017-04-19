extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate fst;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate ndarray;
extern crate protobuf;
extern crate rayon;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tensorflow;
extern crate unicode_normalization;
extern crate yolo;
extern crate zip;

use std::env;
use std::path;

pub use errors::*;
pub use preprocessing::preprocess;
pub use pipeline::slot_filler::Token;
pub use pipeline::intent_parser::IntentClassifierResult;
pub use pipeline::intent_parser::IntentParser;
pub use models::gazetteer::GazetteerKey;

#[cfg(test)]
mod testutils;

pub mod errors;
pub mod config;
pub mod pipeline;
pub mod preprocessing;
mod features;
mod models;
mod postprocessing;
mod protos;

pub fn file_path(file_name: &str) -> path::PathBuf {
    if env::var("DINGHY").is_ok() {
        env::current_exe().unwrap().parent().unwrap().join("test_data/data").join(file_name)
    } else {
        path::PathBuf::from("../data").join(file_name)
    }
}
