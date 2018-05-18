pub mod deterministic_intent_parser;
pub mod probabilistic_intent_parser;

use std::collections::HashSet;

use errors::*;
use snips_nlu_ontology::IntentClassifierResult;

pub use self::deterministic_intent_parser::DeterministicIntentParser;
pub use self::probabilistic_intent_parser::ProbabilisticIntentParser;
pub use slot_utils::InternalSlot;

pub struct InternalParsingResult {
    pub intent: IntentClassifierResult,
    pub slots: Vec<InternalSlot>,
}

pub fn internal_parsing_result(
    intent_name: String,
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
        intents: Option<&HashSet<String>>,
    ) -> Result<Option<InternalParsingResult>>;
}
