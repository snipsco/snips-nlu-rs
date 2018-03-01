mod featurizer;
mod log_reg_intent_classifier;
mod logreg;

use std::collections::HashSet;
use snips_nlu_ontology::IntentClassifierResult;
use errors::*;

pub use self::log_reg_intent_classifier::LogRegIntentClassifier;
pub use self::featurizer::Featurizer;

pub trait IntentClassifier: Send + Sync {
    fn get_intent(
        &self,
        input: &str,
        intents_filter: Option<&HashSet<String>>,
    ) -> Result<Option<IntentClassifierResult>>;
}
