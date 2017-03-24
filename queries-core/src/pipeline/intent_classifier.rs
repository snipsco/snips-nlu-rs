use std::fs;
use std::path;
use std::sync;


use errors::*;
use protobuf;
use ndarray::prelude::*;

use config::IntentConfig;

use protos::model_configuration::ModelConfiguration;
use models::tf::{TensorFlowClassifier, Classifier};
use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};

pub trait IntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability>;
}

pub struct ProtobufIntentClassifier {
    intent_config: sync::Arc<Box<IntentConfig>>,
    intent_model: ModelConfiguration,
    classifier: TensorFlowClassifier,
}

// TODO merge code with protobuf tokens classifier
impl ProtobufIntentClassifier {
    pub fn new(intent_config: sync::Arc<Box<IntentConfig>>, classifier_name: &str) -> Result<ProtobufIntentClassifier> {
        let pb_config = intent_config.get_pb_config()?;
        let model_path = path::Path::new(pb_config.get_intent_classifier_path());
        let mut model_file = intent_config.get_file(model_path)?;
        let intent_model = protobuf::parse_from_reader::<ModelConfiguration>(&mut model_file)?;

        let classifier = TensorFlowClassifier::new(&mut intent_config.get_file(path::Path::new(&intent_model.get_model_path()))?);
        Ok(ProtobufIntentClassifier { intent_config: intent_config.clone(), intent_model: intent_model, classifier: classifier? })
    }
}

impl IntentClassifier for ProtobufIntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(self.intent_config.clone(), &self.intent_model.get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        let probabilities = self.classifier.predict_proba(&computed_features.t());
        Ok(probabilities?[[0, 0]])
    }
}


#[cfg(test)]
mod test {
    use std::fs;

    use ndarray::arr2;

    use preprocessing::preprocess;
    use FileConfiguration;
    use testutils::parse_json;
    use file_path;
    use testutils::create_array;
    use testutils::assert_epsilon_eq;
    use super::{IntentClassifier, ProtobufIntentClassifier};

    #[derive(Deserialize)]
    struct TestDescription {
        text: String,
        output: Vec<Vec<f32>>,
        //        features: Vec<Vec<f32>>,
    }

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn intent_classifier_works() {
        let file_configuration = FileConfiguration::default();
        let paths = fs::read_dir(file_path("snips-sdk-models-protobuf/tests/intent_classification/")).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_name = path.file_stem().unwrap().to_str().unwrap();
            let intent_classifier = ProtobufIntentClassifier::new(&file_configuration, model_name).unwrap();

            for test in tests {
                let preprocess_result = preprocess(&test.text);
                let result = intent_classifier.run(&preprocess_result);
                assert_epsilon_eq(arr2(&[[result]]), create_array(&test.output), 1e-6);
            }
        }
    }
}
