mod featurizer;
mod log_reg_intent_classifier;
mod logreg;

use errors::*;
use snips_nlu_ontology::IntentClassifierResult;
use std::collections::HashSet;

pub use self::featurizer::Featurizer;
pub use self::log_reg_intent_classifier::LogRegIntentClassifier;

pub trait IntentClassifier: Send + Sync {
    fn get_intent(
        &self,
        input: &str,
        intents_filter: Option<&HashSet<String>>,
    ) -> Result<Option<IntentClassifierResult>>;
}
