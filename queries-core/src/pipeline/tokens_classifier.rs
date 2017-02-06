use std::path::Path;

use ndarray::prelude::*;

use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};
use models::model::Model;
use models::cnn::{CNN, TensorflowCNN};

pub trait TokensClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Array2<Probability>;
}

pub struct ProtobufTokensClassifier<'a> {
    model: &'a Model,
}

impl<'a> ProtobufTokensClassifier<'a> {
    pub fn new(model: &'a Model) -> ProtobufTokensClassifier<'a> {
        ProtobufTokensClassifier { model: model }
    }
}

impl<'a> TokensClassifier for ProtobufTokensClassifier<'a> {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Array2<Probability> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(&self.model.get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);

        let model_path = Path::new("../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb");
        let mut cnn = TensorflowCNN::new(model_path);

        let probabilities = cnn.run(&computed_features);

        probabilities
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

    #[test]
    #[ignore]
    fn tokens_classifier_works() {
        let model_directory = "../data/snips-sdk-models-protobuf/tokens_classification/";

        let model_path = Path::new(model_directory)
            .join("BookRestaurant_CnnCrf")
            .with_extension("pbbin");
        let mut model_file = File::open(model_path).unwrap();
        let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();

        let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

        let tokens_classifier = ProtobufTokensClassifier::new(&model);
        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {}", probabilities);
        println!("shape: {:?}", probabilities.shape());
    }
}
