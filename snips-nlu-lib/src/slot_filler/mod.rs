pub mod crf_slot_filler;
mod crf_utils;
mod feature_processor;
mod features;
mod features_utils;

use errors::*;

pub use self::crf_slot_filler::*;
use self::crf_utils::TaggingScheme;
use nlu_utils::token::Token;
use slot_utils::InternalSlot;

pub trait SlotFiller: Send + Sync {
    fn get_tagging_scheme(&self) -> TaggingScheme;
    fn get_slots(&self, text: &str) -> Result<Vec<InternalSlot>>;
    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64>;
}
