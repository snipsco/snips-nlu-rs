use std::fs;

use protobuf;
use ndarray::prelude::*;

use errors::*;
use FileConfiguration;
use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{ MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor };
use models::model::Model;
use models::cnn::{ CNN, TensorflowCNN };

pub trait TokensClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Array2<Probability>>;
}

pub struct ProtobufTokensClassifier {
    file_configuration: FileConfiguration,
    intent_model: Model,
    cnn: TensorflowCNN,
}

impl ProtobufTokensClassifier {
    pub fn new(file_configuration: &FileConfiguration, intent_model_name: &str, cnn_model_name: &str) -> Result<ProtobufTokensClassifier> {
        let model_path = file_configuration.tokens_classifier_path(intent_model_name);
        let mut model_file = fs::File::open(model_path)?;
        let intent_model = protobuf::parse_from_reader::<Model>(&mut model_file)?;

        let cnn_path = file_configuration.tokens_classifier_path(cnn_model_name);
        let cnn = TensorflowCNN::new(cnn_path);

        Ok(ProtobufTokensClassifier { file_configuration: file_configuration.clone(), intent_model: intent_model, cnn: cnn? })
    }
}

impl TokensClassifier for ProtobufTokensClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Array2<Probability>> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(&self.file_configuration, self.intent_model.get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        Ok(self.cnn.run(&computed_features)?)
    }
}

#[cfg(test)]
mod test {
    use preprocessing::preprocess;
    use FileConfiguration;
    use super::{ TokensClassifier, ProtobufTokensClassifier };

    #[test]
    fn tokens_classifier_works() {
        let file_configuration = FileConfiguration::default();
        let model_name = "BookRestaurant_features";
        let cnn_name = "BookRestaurant_model";

        let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

        let tokens_classifier = ProtobufTokensClassifier::new(&file_configuration, model_name, cnn_name).unwrap();
        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {:?}", probabilities);
        println!("shape: {:?}", probabilities.unwrap().shape());
    }
}
