use std::fs;

use errors::*;
use protobuf;
use ndarray::prelude::*;

use FileConfiguration;
use protos::model_configuration::ModelConfiguration;
use models::tf::{TensorFlowClassifier, Classifier};
use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};

pub trait IntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability>;
}

pub struct ProtobufIntentClassifier {
    file_configuration: FileConfiguration,
    model: ModelConfiguration,
}

impl ProtobufIntentClassifier {
    pub fn new(file_configuration: &FileConfiguration, classifier_name: &str) -> Result<ProtobufIntentClassifier> {
        let classifier_path = file_configuration.intent_classifier_path(classifier_name);
        let mut model_file = fs::File::open(classifier_path)?;
        let model = protobuf::parse_from_reader::<ModelConfiguration>(&mut model_file)?;

        Ok(ProtobufIntentClassifier { file_configuration: file_configuration.clone(), model: model })
    }
}

impl IntentClassifier for ProtobufIntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(&self.file_configuration, &self.model.get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);


        let classifier = TensorFlowClassifier::new(&self.file_configuration.intent_classifier_path(&self.model.get_model_path()));


        //let classifier = panic!();
            //LogisticRegression::new(self.model.get_arguments()[0].get_matrix().to_array2());
        let probabilities = classifier?.run(&computed_features);

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
