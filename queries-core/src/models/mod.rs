use std::fs;

use errors::*;
use protobuf;

use FileConfiguration;
use ::pipeline::intent_classifier::ProtobufIntentClassifier;
use ::pipeline::tokens_classifier::ProtobufTokensClassifier;

pub mod gazetteer;
pub mod classifiers;
pub mod model;
pub mod cnn;

pub struct IntentConfiguration {
    pub intent_classifier: ProtobufIntentClassifier,
    pub tokens_classifier: ProtobufTokensClassifier,
    pub slot_names: Vec<String>,
    pub intent_name: String,
}

impl IntentConfiguration {
    pub fn new(file_configuration: &FileConfiguration, intent_name: &str) -> Result<IntentConfiguration> {
        let mut model_file = fs::File::open(file_configuration.configuration_path(intent_name))?;
        let data = protobuf::parse_from_reader::<model::Configuration>(&mut model_file)?;
        let slots = data.get_slots().iter().map(|s| s.get_name().to_string()).collect();

        Ok(IntentConfiguration {
            intent_classifier: Self::build_intent_classifier(file_configuration, &data)?,
            tokens_classifier: Self::build_tokens_classifier(file_configuration, &data)?,
            intent_name: data.intent_name.clone(),
            slot_names: slots,
        })
    }

    fn build_intent_classifier(file_configuration: &FileConfiguration, data: &model::Configuration) -> Result<ProtobufIntentClassifier> {
        ProtobufIntentClassifier::new(file_configuration,
                                      data.get_intent_classifier_name())
    }

    fn build_tokens_classifier(file_configuration: &FileConfiguration, data: &model::Configuration) -> Result<ProtobufTokensClassifier> {
        ProtobufTokensClassifier::new(file_configuration,
                                      data.get_tokens_classifier_name(),
                                      &format!("Cnn_{}", data.get_tokens_classifier_name()))
    }
}
