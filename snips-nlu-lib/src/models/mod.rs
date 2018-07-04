pub mod assistant;
pub mod intent_classifier;
pub mod intent_parser;
pub mod nlu_engine;
pub mod slot_filler;
pub mod processing_unit_metadata;

pub use self::assistant::*;
pub use self::intent_classifier::*;
pub use self::intent_parser::*;
pub use self::nlu_engine::*;
pub use self::slot_filler::*;
pub use self::processing_unit_metadata::*;
use std::path::Path;
use errors::*;

pub trait FromPath {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized;
}
