use errors::*;

use pipeline::intent_classifier::ProtobufIntentClassifier;
use pipeline::tokens_classifier::ProtobufTokensClassifier;
use config::ArcBoxedIntentConfig;

pub mod gazetteer;
pub mod tf;

pub struct IntentConfiguration {
    pub intent_classifier: ProtobufIntentClassifier,
    pub tokens_classifier: ProtobufTokensClassifier,
    pub slot_names: Vec<String>,
    pub intent_name: String,
}

impl IntentConfiguration {
    pub fn new(intent_config: ArcBoxedIntentConfig) -> Result<IntentConfiguration> {
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
