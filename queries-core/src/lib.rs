extern crate itertools;

extern crate rustc_serialize;
#[macro_use(stack)]
extern crate ndarray;

extern crate unicode_normalization;

extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate protobuf;

pub mod models;
pub mod preprocessing;
pub mod pipeline;

#[cfg(test)]
mod testutils;
