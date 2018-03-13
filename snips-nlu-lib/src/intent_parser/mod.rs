pub mod deterministic_intent_parser;
pub mod probabilistic_intent_parser;

use std::collections::HashSet;

use errors::*;
use snips_nlu_ontology::IntentClassifierResult;

pub use self::deterministic_intent_parser::DeterministicIntentParser;
pub use self::probabilistic_intent_parser::ProbabilisticIntentParser;
pub use slot_utils::InternalSlot;

pub trait IntentParser: Send + Sync {
    fn get_intent(
        &self,
        input: &str,
        intents: Option<&HashSet<String>>,
    ) -> Result<Option<IntentClassifierResult>>;
    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<InternalSlot>>;
}
