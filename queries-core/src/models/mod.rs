use std::{ path, fs };

use protobuf;

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
    pub fn new<P: AsRef<path::Path>>(path: P) -> IntentConfiguration {
        let mut model_file = fs::File::open(path).unwrap();
        let data = protobuf::parse_from_reader::<model::Configuration>(&mut model_file).unwrap();
        let slots = data.get_slots().iter().map(|s| s.get_name().to_string()).collect();
        IntentConfiguration {
            intent_classifier: Self::build_intent_classifier(&data),
            tokens_classifier: Self::build_tokens_classifier(&data),
            intent_name: data.intent_name.clone(),
            slot_names: slots,
        }
    }

    fn build_intent_classifier(data: &model::Configuration) -> ProtobufIntentClassifier {
        ProtobufIntentClassifier::new(data.get_intent_classifier_name())
    }

    fn build_tokens_classifier(data: &model::Configuration) -> ProtobufTokensClassifier {
        ProtobufTokensClassifier::new(data.get_tokens_classifier_name(),
                                      format!("Cnn_{}", data.get_tokens_classifier_name()))
    }
}
