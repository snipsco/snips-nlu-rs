use std::path;

use errors::*;
use protobuf;
use ndarray::prelude::*;

use super::BoxedClassifier;
use super::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};
use config::ArcBoxedIntentConfig;
use preprocessing::PreprocessorResult;
use postprocessing;
use protos::model_configuration::ModelConfiguration;
use models::tf::{TensorFlowClassifier, TensorFlowCRFClassifier};

pub trait TokensClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Array1<usize>>;
}

pub struct ProtobufTokensClassifier {
    intent_config: ArcBoxedIntentConfig,
    intent_model: ModelConfiguration,
    classifier: BoxedClassifier,
}


// TODO merge code with protobuf intent classifier
impl ProtobufTokensClassifier {
    pub fn new(intent_config: ArcBoxedIntentConfig) -> Result<ProtobufTokensClassifier> {
        let pb_config = intent_config.get_pb_config()?;
        let model_path = path::Path::new(pb_config.get_tokens_classifier_path());
        let mut model_file = intent_config.get_file(model_path)?;
        let intent_model = protobuf::parse_from_reader::<ModelConfiguration>(&mut model_file)?;
        let tf_model =
            &mut intent_config.get_file(path::Path::new(&intent_model.get_model_path()))?;
        let classifier: BoxedClassifier = if intent_model.has_crf {
            Box::new(TensorFlowCRFClassifier::new(tf_model,
                                                  pb_config.get_slots().len() as u32,
                                                  intent_model.get_input_node().to_string(),
                                                  intent_model.get_output_node().to_string(),
                                                  intent_model.get_transition_matrix_node())?)
        } else {
            Box::new(TensorFlowClassifier::new(tf_model,
                                               intent_model.get_input_node().to_string(),
                                               intent_model.get_output_node().to_string())?)
        };

        Ok(ProtobufTokensClassifier {
            intent_config: intent_config.clone(),
            intent_model: intent_model,
            classifier: classifier,
        })
    }
}

impl TokensClassifier for ProtobufTokensClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Array1<usize>> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(self.intent_config.clone(),
                                                                    self.intent_model
                                                                        .get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        let predictions = self.classifier.predict(&computed_features.t())?;

        if self.intent_model.has_bilou {
            Ok(postprocessing::bilou(predictions))
        } else {
            Ok(predictions)
        }
    }
}

#[cfg(test)]
mod test {
    use config::{AssistantConfig, FileBasedAssistantConfig};
    use preprocessing::preprocess;
    use super::{TokensClassifier, ProtobufTokensClassifier};

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn tokens_classifier_works() {
        let preprocessor_result = preprocess("Book me a table for two people at Le Chalet \
                                              Savoyard", "").unwrap();

        let intent_config = FileBasedAssistantConfig::default().get_intent_configuration("BookRestaurant").unwrap();

        let tokens_classifier = ProtobufTokensClassifier::new(intent_config).unwrap();

        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {:?}", probabilities);
        println!("shape: {:?}", probabilities.unwrap().shape());
    }
}
