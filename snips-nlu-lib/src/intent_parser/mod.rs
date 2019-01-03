pub mod deterministic_intent_parser;
pub mod probabilistic_intent_parser;

use std::path::Path;
use std::sync::Arc;

use snips_nlu_ontology::IntentClassifierResult;

pub use self::deterministic_intent_parser::DeterministicIntentParser;
pub use self::probabilistic_intent_parser::ProbabilisticIntentParser;
use crate::errors::*;
use crate::models::ProcessingUnitMetadata;
use crate::resources::SharedResources;
use crate::utils::IntentName;
pub use crate::slot_utils::InternalSlot;

pub struct InternalParsingResult {
    pub intent: IntentClassifierResult,
    pub slots: Vec<InternalSlot>,
}

pub fn internal_parsing_result(
    intent_name: IntentName,
    intent_proba: f32,
    slots: Vec<InternalSlot>,
) -> InternalParsingResult {
    InternalParsingResult {
        intent: IntentClassifierResult {
            intent_name,
            probability: intent_proba,
        },
        slots,
    }
}

pub trait IntentParser: Send + Sync {
    fn parse(
        &self,
        input: &str,
        intents: Option<&[&str]>,
    ) -> Result<Option<InternalParsingResult>>;

    fn get_slots(
        &self,
        input: &str,
        intent: &str
    ) -> Result<Vec<InternalSlot>>;
}

pub fn build_intent_parser<P: AsRef<Path>>(
    metadata: ProcessingUnitMetadata,
    path: P,
    shared_resources: Arc<SharedResources>,
) -> Result<Box<IntentParser>> {
    match metadata {
        ProcessingUnitMetadata::DeterministicIntentParser => Ok(Box::new(
            DeterministicIntentParser::from_path(path, shared_resources)?,
        ) as _),
        ProcessingUnitMetadata::ProbabilisticIntentParser => Ok(Box::new(
            ProbabilisticIntentParser::from_path(path, shared_resources)?,
        ) as _),
        _ => Err(format_err!("{:?} is not an intent parser", metadata)),
    }
}
