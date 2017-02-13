use std::{ fs, path };

use protobuf;
use ndarray::prelude::*;

use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};
use models::model::Model;
use models::cnn::{ CNN, TensorflowCNN };

pub trait TokensClassifier {
    fn run(&mut self, preprocessor_result: &PreprocessorResult) -> Array2<Probability>;
}

pub struct ProtobufTokensClassifier {
    intent_model: Model,
    classifier: TensorflowCNN,
}

impl ProtobufTokensClassifier {
    pub fn new<P1, P2>(intent_model_path: P1, classifier_path: P2) -> ProtobufTokensClassifier
            where P1: AsRef<path::Path>, P2: AsRef<path::Path> {
        let mut model_file = fs::File::open(intent_model_path).unwrap();
        let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();
        let classifier = TensorflowCNN::new(classifier_path.as_ref());
        ProtobufTokensClassifier { intent_model: model, classifier: classifier }
    }
}

impl TokensClassifier for ProtobufTokensClassifier {
    fn run(&mut self, preprocessor_result: &PreprocessorResult) -> Array2<Probability> {
        let feature_processor =  ProtobufMatrixFeatureProcessor::new(self.intent_model.get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        self.classifier.run(&computed_features)
    }
}

#[cfg(test)]
mod test {
    extern crate protobuf;

    use std::fs::File;
    use std::path::Path;
    use models::model::Model;
    use preprocessing::preprocess;
    use super::{TokensClassifier, ProtobufTokensClassifier};
    use models::cnn::TensorflowCNN;

    #[test]
    #[ignore]
    fn tokens_classifier_works() {
        let model_directory = "../data/snips-sdk-models-protobuf/tokens_classification/";
        let cnn_path = "../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb";

        let model_path = Path::new(model_directory)
            .join("BookRestaurant_CnnCrf")
            .with_extension("pbbin");
        let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

        let mut tokens_classifier = ProtobufTokensClassifier::new(model_path, cnn_path);
        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {}", probabilities);
        println!("shape: {:?}", probabilities.shape());
    }
}
