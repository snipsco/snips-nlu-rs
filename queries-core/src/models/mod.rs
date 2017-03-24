use std::sync;

use errors::*;

use config::IntentConfig;
use pipeline::intent_classifier::ProtobufIntentClassifier;
use pipeline::tokens_classifier::ProtobufTokensClassifier;

pub mod gazetteer;
pub mod classifiers;
pub mod tf;

pub struct IntentConfiguration {
    pub intent_classifier: ProtobufIntentClassifier,
    pub tokens_classifier: ProtobufTokensClassifier,
    pub slot_names: Vec<String>,
    pub intent_name: String,
}

unsafe impl Sync for IntentConfiguration {}

impl IntentConfiguration {
    pub fn new(intent_config: sync::Arc<Box<IntentConfig>>) -> Result<IntentConfiguration> {
        let data = intent_config.get_pb_config()?;
        let slots = data.get_slots().iter().map(|s| s.get_name().to_string()).collect();

        Ok(IntentConfiguration {
            intent_classifier: ProtobufIntentClassifier::new(intent_config.clone())?,
            tokens_classifier: ProtobufTokensClassifier::new(intent_config.clone())?,
            intent_name: data.name.clone(),
            slot_names: slots,
        })
    }
}
