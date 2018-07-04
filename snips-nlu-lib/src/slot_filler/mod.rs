pub mod crf_slot_filler;
mod crf_utils;
mod feature_processor;
mod features;
mod features_utils;

use std::fs::File;
use std::path::Path;

use errors::*;
use serde_json;

pub use self::crf_slot_filler::*;
use self::crf_utils::TaggingScheme;
use models::{FromPath, ProcessingUnitMetadata};
use nlu_utils::token::Token;
use slot_utils::InternalSlot;

pub trait SlotFiller: FromPath + Send + Sync {
    fn get_tagging_scheme(&self) -> TaggingScheme;
    fn get_slots(&self, text: &str) -> Result<Vec<InternalSlot>>;
    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64>;
}

pub fn build_slot_filler<P: AsRef<Path>>(path: P) -> Result<Box<SlotFiller>> {
    let metadata_path = path.as_ref().join("metadata.json");
    let metadata_file = File::open(metadata_path)?;
    let metadata: ProcessingUnitMetadata = serde_json::from_reader(metadata_file)?;
    match metadata {
        ProcessingUnitMetadata::CrfSlotFiller => Ok(Box::new(CRFSlotFiller::from_path(path)?) as _),
        _ => Err(format_err!("{:?} is not a slot filler", metadata))
    }
}
