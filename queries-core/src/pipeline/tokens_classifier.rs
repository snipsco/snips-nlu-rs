use std::{ fs, path };

use protobuf;
use ndarray::prelude::*;

use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};
use models::model::Model;
use models::cnn::CNN;

pub trait TokensClassifier {
    fn run(&mut self, preprocessor_result: &PreprocessorResult) -> Array2<Probability>;
}

pub struct ProtobufTokensClassifier<'a, C:CNN+'a> {
    intent_model: Model,
    classifier: &'a mut C,
}

impl<'a, C:CNN> ProtobufTokensClassifier<'a, C> {
    pub fn new<P: AsRef<path::Path>>(intent_model: P, classifier: &'a mut C) -> ProtobufTokensClassifier<'a, C> {
        let mut model_file = fs::File::open(intent_model).unwrap();
        let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();
        ProtobufTokensClassifier { intent_model: model, classifier: classifier }
    }
}

impl<'a, C:CNN> TokensClassifier for ProtobufTokensClassifier<'a, C> {
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
        let model_path = Path::new("../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb");

        let mut cnn = TensorflowCNN::new(model_path);

        let model_path = Path::new(model_directory)
            .join("BookRestaurant_CnnCrf")
            .with_extension("pbbin");
        let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

        let tokens_classifier = ProtobufTokensClassifier::new(model_path, &mut cnn);
        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {}", probabilities);
        println!("shape: {:?}", probabilities.shape());
    }
}
