use std::fs;
use std::sync;

use errors::*;
use protobuf;

use config::IntentConfig;
use ::pipeline::intent_classifier::ProtobufIntentClassifier;
use ::pipeline::tokens_classifier::ProtobufTokensClassifier;
use ::protos::intent_configuration::IntentConfiguration as PBIntentConfiguration;

pub mod gazetteer;
pub mod classifiers;
pub mod cnn;
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
            intent_classifier: Self::build_intent_classifier(intent_config.clone(), &data)?,
            tokens_classifier: Self::build_tokens_classifier(intent_config.clone(), &data)?,
            intent_name: data.name.clone(),
            slot_names: slots,
        })
    }

    fn build_intent_classifier(intent_config: sync::Arc<Box<IntentConfig>>, data: &PBIntentConfiguration) -> Result<ProtobufIntentClassifier> {
        ProtobufIntentClassifier::new(intent_config,
                                      data.get_intent_classifier_path())
    }

    fn build_tokens_classifier(intent_config: sync::Arc<Box<IntentConfig>>, data: &PBIntentConfiguration) -> Result<ProtobufTokensClassifier> {
        ProtobufTokensClassifier::new(intent_config,
                                      data.get_tokens_classifier_path())
    }
}
