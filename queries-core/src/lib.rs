extern crate itertools;

extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

#[macro_use(stack)]
extern crate ndarray;

extern crate unicode_normalization;

extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate protobuf;

extern crate rayon;

extern crate rulinalg;

extern crate tensorflow;

pub mod models;
pub mod preprocessing;
pub mod pipeline;
pub mod features;

pub use preprocessing::preprocess;

#[cfg(test)]
mod testutils;
