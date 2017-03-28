use std::path;

use errors::*;
use protobuf;

use config::ArcBoxedIntentConfig;
use protos::model_configuration::ModelConfiguration;
use models::tf::TensorFlowClassifier;
use preprocessing::PreprocessorResult;
use pipeline::Probability;
use pipeline::feature_processor::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};
use pipeline::BoxedClassifier;

pub trait IntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability>;
}

pub struct ProtobufIntentClassifier {
    intent_config: ArcBoxedIntentConfig,
    intent_model: ModelConfiguration,
    classifier: BoxedClassifier,
}

// TODO merge code with protobuf tokens classifier
impl ProtobufIntentClassifier {
    pub fn new(intent_config: ArcBoxedIntentConfig) -> Result<ProtobufIntentClassifier> {
        let pb_config = intent_config.get_pb_config()?;
        let model_path = path::Path::new(pb_config.get_intent_classifier_path());
        let mut model_file = intent_config.get_file(model_path)?;
        let intent_model = protobuf::parse_from_reader::<ModelConfiguration>(&mut model_file)?;
        let tf_model =
            &mut intent_config.get_file(path::Path::new(&intent_model.get_model_path()))?;
        let classifier =
            Box::new(TensorFlowClassifier::new(tf_model,
                                               intent_model.get_input_node().to_string(),
                                               intent_model.get_output_node().to_string())?);
        Ok(ProtobufIntentClassifier {
            intent_config: intent_config.clone(),
            intent_model: intent_model,
            classifier: classifier,
        })
    }
}

impl IntentClassifier for ProtobufIntentClassifier {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability> {
        let feature_processor = ProtobufMatrixFeatureProcessor::new(self.intent_config.clone(),
                                                                    &self.intent_model
                                                                        .get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        let probabilities = self.classifier.predict_proba(&computed_features.t());
        Ok(probabilities?[[0, 0]])
    }
}


#[cfg(test)]
mod test {
    use std::fs;
    use std::path;
    use std::sync;

    use protobuf;
    use ndarray::arr2;

    use file_path;
    use models::IntentConfiguration;
    use config::{AssistantConfig, IntentConfig, FileBasedAssistantConfig};
    use preprocessing::preprocess;
    use protos::model_configuration::ModelConfiguration;
    use testutils::parse_json;
    use testutils::create_array;
    use testutils::assert_epsilon_eq;
    use super::{IntentClassifier, ProtobufIntentClassifier};

    #[derive(Deserialize)]
    struct TestDescription {
        text: String,
        output: Vec<Vec<f32>>,
    }

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn intent_classifier_works() {
        let assistant_config = FileBasedAssistantConfig::default();
        let paths =
            fs::read_dir(file_path("snips-sdk-models-protobuf/tests/intent_classification/"))
                .unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_name = path.file_stem().unwrap().to_str().unwrap();
            let intent_config = FileBasedAssistantConfig::default().get_intent_configuration(model_name).unwrap();
            let pb_intent_config = intent_config.get_pb_config().unwrap();
            let mut intent_classifier_config = intent_config.get_file(path::Path::new(pb_intent_config.get_intent_classifier_path())).unwrap();
            let pb_intent_configuration = protobuf::parse_from_reader::<ModelConfiguration>(&mut intent_classifier_config).unwrap();

            let intent_classifier = ProtobufIntentClassifier::new(intent_config).unwrap();

            for test in tests {
                let preprocess_result = preprocess(&test.text);
                let result = intent_classifier.run(&preprocess_result).unwrap();
                assert_epsilon_eq(&arr2(&[[result]]), &create_array(&test.output), 1e-6);
            }
        }
    }
}
