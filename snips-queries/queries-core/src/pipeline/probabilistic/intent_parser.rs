use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use errors::*;
use pipeline::IntentParser;
use pipeline::probabilistic::configuration::ProbabilisticParserConfiguration;
use pipeline::probabilistic::intent_classifier::{IntentClassifier, LogRegIntentClassifier};
use pipeline::probabilistic::slot_filler::{CRFSlotFiller, SlotFiller};
use snips_queries_ontology::{IntentClassifierResult, Slot};


pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_fillers: HashMap<String, Box<SlotFiller>>,
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserConfiguration) -> Result<Self> {
        let slot_fillers_vec: Result<Vec<_>> = config
            .slot_fillers
            .into_iter()
            .map(|(intent_name, slot_filler_config)| {
                Ok((intent_name, Box::new(CRFSlotFiller::new(slot_filler_config)?) as _))
            })
            .collect();
        let slot_fillers = HashMap::from_iter(slot_fillers_vec?);
        let intent_classifier =
            Box::new(LogRegIntentClassifier::new(config.intent_classifier)?) as _;

        Ok(ProbabilisticIntentParser { intent_classifier, slot_fillers })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn get_intent(&self,
                  input: &str,
                  intents: Option<&HashSet<String>>) -> Result<Option<IntentClassifierResult>> {
        self.intent_classifier.get_intent(input, intents)
    }

    fn get_slots(&self, input: &str, intent_name: &str) -> Result<Vec<Slot>> {
        self.slot_fillers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found in slot fillers", intent_name))?
            .get_slots(input)
    }
}
