use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use models::ProbabilisticParserModel;
use errors::*;
use intent_classifier::{IntentClassifier, LogRegIntentClassifier};
use intent_parser::{IntentParser, InternalParsingResult};
use slot_filler::{CRFSlotFiller, SlotFiller};

pub struct ProbabilisticIntentParser {
    intent_classifier: Box<IntentClassifier>,
    slot_fillers: HashMap<String, Box<SlotFiller>>,
}

impl ProbabilisticIntentParser {
    pub fn new(config: ProbabilisticParserModel) -> Result<Self> {
        let slot_fillers_vec: Result<Vec<_>> = config
            .slot_fillers
            .into_iter()
            .map(|(intent_name, slot_filler_config)| {
                Ok((
                    intent_name,
                    Box::new(CRFSlotFiller::new(slot_filler_config)?) as _,
                ))
            })
            .collect();
        let slot_fillers = HashMap::from_iter(slot_fillers_vec?);
        let intent_classifier =
            Box::new(LogRegIntentClassifier::new(config.intent_classifier)?) as _;

        Ok(ProbabilisticIntentParser {
            intent_classifier,
            slot_fillers,
        })
    }
}

impl IntentParser for ProbabilisticIntentParser {
    fn parse(
        &self,
        input: &str,
        intents: Option<&HashSet<String>>,
    ) -> Result<Option<InternalParsingResult>> {
        let opt_intent_result = self.intent_classifier.get_intent(input, intents)?;
        if let Some(intent_result) = opt_intent_result {
            let slots = self.slot_fillers
                .get(&*intent_result.intent_name)
                .ok_or_else(|| {
                    format_err!(
                        "intent {} not found in slot fillers",
                        intent_result.intent_name
                    )
                })?
                .get_slots(input)?;
            Ok(Some(InternalParsingResult {
                intent: intent_result,
                slots,
            }))
        } else {
            Ok(None)
        }
    }
}
