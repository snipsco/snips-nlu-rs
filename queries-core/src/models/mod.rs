use std::fs;

use protobuf;

use ::FileConfiguration;
use ::pipeline::intent_classifier::ProtobufIntentClassifier;
use ::pipeline::tokens_classifier::ProtobufTokensClassifier;

pub mod gazetteer;
pub mod classifiers;
pub mod model;
pub mod cnn;

pub struct IntentConfiguration<'a> {
    pub intent_classifier: ProtobufIntentClassifier<'a>,
    pub tokens_classifier: ProtobufTokensClassifier,
    pub slot_names: Vec<String>,
    pub intent_name: String,
}

impl<'a> IntentConfiguration<'a> {
    pub fn new(file_configuration: &'a FileConfiguration, intent_name: &str) -> IntentConfiguration<'a> {
        let mut model_file = fs::File::open(file_configuration.configuration_path(intent_name)).unwrap();
        let data = protobuf::parse_from_reader::<model::Configuration>(&mut model_file).unwrap();
        let slots = data.get_slots().iter().map(|s| s.get_name().to_string()).collect();
        IntentConfiguration {
            intent_classifier: Self::build_intent_classifier(file_configuration, &data),
            tokens_classifier: Self::build_tokens_classifier(file_configuration, &data),
            intent_name: data.intent_name.clone(),
            slot_names: slots,
        }
    }

    fn build_intent_classifier(file_configuration: &'a FileConfiguration, data: &model::Configuration) -> ProtobufIntentClassifier<'a> {
        ProtobufIntentClassifier::new(file_configuration,
                                      data.get_intent_classifier_name())
    }

    fn build_tokens_classifier(file_configuration: &FileConfiguration, data: &model::Configuration) -> ProtobufTokensClassifier {
        ProtobufTokensClassifier::new(file_configuration,
                                      data.get_tokens_classifier_name(),
                                      &format!("Cnn_{}", data.get_tokens_classifier_name())).unwrap()
    }
}
