use errors::*;
use ndarray::prelude::*;

use pipeline::FeatureProcessor;
use config::ArcBoxedIntentConfig;
use config::IntentConfig;
use features::{shared_scalar, shared_vector};
use preprocessing::PreprocessorResult;
use protos::feature::{Feature, Feature_Scalar_Type, Feature_Vector_Type, Feature_Argument};

pub struct DeepFeatureProcessor<'a> {
    intent_config: ArcBoxedIntentConfig,
    feature_functions: &'a [Feature],
}

impl<'a> DeepFeatureProcessor<'a> {
    pub fn new(intent_config: ArcBoxedIntentConfig,
               features: &'a [Feature])
               -> DeepFeatureProcessor<'a> {
        DeepFeatureProcessor {
            intent_config: intent_config,
            feature_functions: features,
        }
    }
}

impl<'a> FeatureProcessor<PreprocessorResult, Array2<f32>> for DeepFeatureProcessor<'a> {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f32> {
        let computed_values = self.feature_functions
            .iter()
            .flat_map(|feature_function| feature_function.compute(&**self.intent_config, input).unwrap()) // TODO: Dunno how to kill this unwrap
            .collect::<Vec<f32>>();

        let len = self.feature_functions.len();
        let feature_length = computed_values.len() / len;

        match Array::from_vec(computed_values).into_shape((len, feature_length)) {
            Ok(array) => array,
            Err(_) => panic!("A feature function doesn't have the same len as the others."),
        }
    }
}

impl Feature {
    fn compute(&self, intent_config: &IntentConfig, input: &PreprocessorResult) -> Result<Vec<f32>> {
        let arguments = self.get_arguments();

        if self.has_scalar_type() {
            Self::get_shared_scalar(intent_config, input, &self.get_scalar_type(), arguments)
        } else if self.has_vector_type() {
            Self::get_shared_vector(intent_config, input, &self.get_vector_type(), arguments)
        } else {
            bail!("No feature function passed")
        }
    }

    fn get_shared_scalar(intent_config: &IntentConfig,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Scalar_Type,
                         arguments: &[Feature_Argument])
                         -> Result<Vec<f32>> {
        Ok(match *feature_type {
            Feature_Scalar_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = intent_config.get_gazetteer(arguments[0].get_gazetteer())?;
                shared_scalar::has_gazetteer_hits(input, gazetteer)
            }
            Feature_Scalar_Type::NGRAM_MATCHER => {
                shared_scalar::ngram_matcher(input, arguments[0].get_str())
            }
            Feature_Scalar_Type::GET_MESSAGE_LENGTH => {
                let normalization = arguments[0].get_scalar();
                shared_scalar::get_message_length(input, normalization)
            }
        })
    }

    fn get_shared_vector(intent_config: &IntentConfig,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Vector_Type,
                         arguments: &[Feature_Argument])
                         -> Result<Vec<f32>> {
        Ok(match *feature_type {
            Feature_Vector_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = intent_config.get_gazetteer(arguments[0].get_gazetteer())?;
                shared_vector::has_gazetteer_hits(input, gazetteer)
            }
            Feature_Vector_Type::NGRAM_MATCHER => {
                shared_vector::ngram_matcher(input, arguments[0].get_str())
            }
            Feature_Vector_Type::IS_CAPITALIZED => shared_vector::is_capitalized(input),
            Feature_Vector_Type::IS_FIRST_WORD => shared_vector::is_first_word(input),
            Feature_Vector_Type::IS_LAST_WORD => shared_vector::is_last_word(input),
            Feature_Vector_Type::CONTAINS_POSSESSIVE => shared_vector::contains_possessive(input),
            Feature_Vector_Type::CONTAINS_DIGITS => shared_vector::contains_digits(input),
        })
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path;

    use protobuf;

    use file_path;
    use config::{AssistantConfig, FileBasedAssistantConfig};
    use preprocessing::preprocess;
    use protos::model_configuration::ModelConfiguration;
    use testutils::parse_json;
    use testutils::create_transposed_array;
    use super::{FeatureProcessor, DeepFeatureProcessor};

    #[derive(Deserialize, Debug)]
    struct TestDescription {
        text: String,
        entities: String,
        version: String,
        #[serde(rename = "type")]
        kind: String,
        features: Vec<Vec<f32>>,
        //output: Vec<Vec<f32>>,
    }

    #[test]
    fn feature_processor_works() {
        let paths_result = fs::read_dir(file_path("untracked/tests/"));
        if let Err(_) = paths_result {
            return;
        }
        let assistant_config = FileBasedAssistantConfig::new(file_path("untracked")).unwrap();

        for entry in paths_result.unwrap() {
            let path = entry.unwrap().path();
            if !path.is_dir() {
                continue;
            }

            let model_name = path.file_stem().unwrap().to_str().unwrap();
            let intent_config = assistant_config
                .get_intent_configuration(model_name)
                .unwrap();
            let pb_intent_config = intent_config.get_pb_config().unwrap();

            let test = |test_filename, pb_model_config: ModelConfiguration| {
                let tests: Vec<TestDescription> =
                    parse_json(path.join(test_filename).to_str().unwrap());
                for test in tests {
                    let preprocessor_result = preprocess(&test.text, &test.entities).unwrap();

                    let feature_processor =
                        DeepFeatureProcessor::new(intent_config.clone(),
                                                  &pb_model_config.get_features());
                    let result = feature_processor.compute_features(&preprocessor_result);

                    let formatted_errors: Vec<String> = create_transposed_array(&test.features).genrows().into_iter().enumerate()
                        .filter_map(|(i, expected_row)| {
                            let retrieved_row = result.row(i);
                            if retrieved_row != expected_row {
                                Some(format!("feature #{} failed - ({:?}).\n\tgot:      {}\n\texpected: {}",
                                i, &pb_model_config.get_features()[i], retrieved_row, expected_row))
                            } else {
                                None
                            }
                        })
                    .collect();

                    assert!(formatted_errors.len() == 0, "{} {} v{}, input: {}\n{}",
                        &test.kind, model_name, &test.version, &test.text, formatted_errors.join("\n"));
                }
            };

            {
                let mut classifier_config = intent_config.get_file(path::Path::new(pb_intent_config.get_intent_classifier_path())).unwrap();
                let pb_model_config: ModelConfiguration = protobuf::parse_from_reader(&mut classifier_config).unwrap();
                test("intent_classifier.json", pb_model_config);
            }
            {
                let mut classifier_config = intent_config.get_file(path::Path::new(pb_intent_config.get_tokens_classifier_path())).unwrap();
                let pb_model_config: ModelConfiguration = protobuf::parse_from_reader(&mut classifier_config).unwrap();
                test("entity_extraction.json", pb_model_config);
            }
        }
    }
}
