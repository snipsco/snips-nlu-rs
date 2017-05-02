use std::marker::PhantomData;
use std::path;
use std::io::Read;

use errors::*;
use protobuf;
use ndarray::prelude::*;

use pipeline::{Probability, Prediction};
use pipeline::FeatureProcessor;
use super::feature_processor::DeepFeatureProcessor;
use pipeline::BoxedClassifier;
use pipeline::ClassifierWrapper;
use config::ArcBoxedIntentConfig;
use protos::model_configuration::ModelConfiguration;
use protos::intent_configuration::IntentConfiguration;
use models::tf::{TensorFlowClassifier, TensorFlowCRFClassifier};
use preprocessing::PreprocessorResult;
use postprocessing;

pub struct TFClassifierWrapper<T> {
    intent_config: ArcBoxedIntentConfig,
    intent_model: ModelConfiguration,
    classifier: BoxedClassifier,
    phantom: PhantomData<T>
}

impl TFClassifierWrapper<TFClassifierWrapper<Probability>> {
    pub fn new_intent_classifier(intent_config: ArcBoxedIntentConfig) -> Result<TFClassifierWrapper<Probability>> {
        let (mut tf_model, intent_model) = get_model_file(&intent_config,
                                                          |pb_config| pb_config.get_intent_classifier_path())?;
        let classifier =
            Box::new(TensorFlowClassifier::new(&mut *tf_model,
                                               intent_model.get_input_node().to_string(),
                                               intent_model.get_output_node().to_string())?);
        Ok(TFClassifierWrapper {
            intent_config: intent_config,
            intent_model: intent_model,
            classifier: classifier,
            phantom: PhantomData
        })
    }
}

impl TFClassifierWrapper<TFClassifierWrapper<Array1<Prediction>>> {
    pub fn new_tokens_classifier(intent_config: ArcBoxedIntentConfig) -> Result<TFClassifierWrapper<Array1<Prediction>>> {
        let (mut tf_model, intent_model) = get_model_file(&intent_config,
                                                          |pb_config| pb_config.get_tokens_classifier_path())?;
        let classifier: BoxedClassifier = if intent_model.has_crf {
            Box::new(TensorFlowCRFClassifier::new(&mut *tf_model,
                                                  intent_config.get_pb_config()?.get_slots().len() as u32, // TODO avoid get_pb_config twice
                                                  intent_model.get_input_node().to_string(),
                                                  intent_model.get_output_node().to_string(),
                                                  intent_model.get_transition_matrix_node())?)
        } else {
            Box::new(TensorFlowClassifier::new(&mut *tf_model,
                                               intent_model.get_input_node().to_string(),
                                               intent_model.get_output_node().to_string())?)
        };

        Ok(TFClassifierWrapper {
            intent_config: intent_config.clone(),
            intent_model: intent_model,
            classifier: classifier,
            phantom: PhantomData
        })
    }
}

fn get_model_file<F>(intent_config: &ArcBoxedIntentConfig, path_extractor: F) -> Result<(Box<Read>, ModelConfiguration)>
    where F: Fn(&IntentConfiguration) -> &str {
    let pb_config = intent_config.get_pb_config()?;
    let path_str = path_extractor(&pb_config);
    let model_path = path::Path::new(&path_str);
    let mut model_file = intent_config.get_file(&model_path)?;
    let model_configuration = protobuf::parse_from_reader::<ModelConfiguration>(&mut model_file)?;
    let tf_model = intent_config.get_file(path::Path::new(&model_configuration.get_model_path()))?;
    Ok((tf_model, model_configuration))
}

impl ClassifierWrapper<PreprocessorResult, Probability> for TFClassifierWrapper<Probability> {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Probability> {
        let feature_processor = DeepFeatureProcessor::new(self.intent_config.clone(),
                                                                    &self.intent_model
                                                                        .get_features());
        let computed_features = feature_processor.compute_features(preprocessor_result);
        let probabilities = self.classifier.predict_proba(&computed_features.t());
        Ok(probabilities?[[0, 0]])
    }
}

impl ClassifierWrapper<PreprocessorResult, Array1<Prediction>> for TFClassifierWrapper<Array1<Prediction>> {
    fn run(&self, preprocessor_result: &PreprocessorResult) -> Result<Array1<usize>> {
        let feature_processor = DeepFeatureProcessor::new(self.intent_config.clone(),
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
    use std::fs;

    use ndarray::arr2;

    use config::{AssistantConfig, FileBasedAssistantConfig};
    use preprocessing::{DeepPreprocessor, Preprocessor};
    use testutils::{create_array, assert_epsilon_eq};
    use utils::{file_path, parse_json};
    use super::{ClassifierWrapper, TFClassifierWrapper};

    #[derive(Deserialize)]
    struct TestDescription {
        text: String,
        output: Vec<Vec<f32>>,
    }

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn intent_classifier_works() {
        let paths =
            fs::read_dir(file_path("snips-sdk-models-protobuf/tests/intent_classification/"))
                .unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_name = path.file_stem().unwrap().to_str().unwrap();
            let intent_config = FileBasedAssistantConfig::default().get_intent_configuration(model_name).unwrap();

            let intent_classifier = TFClassifierWrapper::new_intent_classifier(intent_config).unwrap();

            let preprocessor = DeepPreprocessor::new("en").unwrap();
            for test in tests {
                // TODO: Build PreprocessorResult from test instead of running the preprocessor
                let preprocess_result = preprocessor.run(&test.text).unwrap();
                let result = intent_classifier.run(&preprocess_result).unwrap();
                assert_epsilon_eq(&arr2(&[[result]]), &create_array(&test.output), 1e-6);
            }
        }
    }

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn tokens_classifier_works() {
        let preprocessor = DeepPreprocessor::new("en").unwrap();
        let preprocessor_result = preprocessor.run("Book me a table for two people at Le Chalet Savoyard").unwrap();

        let intent_config = FileBasedAssistantConfig::default().get_intent_configuration("BookRestaurant").unwrap();

        let tokens_classifier = TFClassifierWrapper::new_tokens_classifier(intent_config).unwrap();

        let probabilities = tokens_classifier.run(&preprocessor_result);

        println!("probabilities: {:?}", probabilities);
        println!("shape: {:?}", probabilities.unwrap().shape());
    }
}
