use errors::*;
use ndarray::{Array, Array2};
use std::sync;

use preprocessing::PreprocessorResult;
use protos::feature::{Feature, Feature_Type, Feature_Domain, Feature_Argument};

use config::IntentConfig;

pub trait MatrixFeatureProcessor {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f32>;
}

pub struct ProtobufMatrixFeatureProcessor<'a> {
    intent_config: sync::Arc<Box<IntentConfig>>,
    feature_functions: &'a [Feature],
}

impl<'a> ProtobufMatrixFeatureProcessor<'a> {
    pub fn new(intent_config: sync::Arc<Box<IntentConfig>>,
               features: &'a [Feature])
               -> ProtobufMatrixFeatureProcessor<'a> {
        ProtobufMatrixFeatureProcessor {
            intent_config: intent_config,
            feature_functions: features,
        }
    }
}

impl<'a> MatrixFeatureProcessor for ProtobufMatrixFeatureProcessor<'a> {
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
    fn compute(&self,
               intent_config: &IntentConfig,
               input: &PreprocessorResult)
               -> Result<Vec<f32>> {
        let known_domain = self.get_domain();
        let feature_type = self.field_type;
        let arguments = self.get_arguments();

        match known_domain {
            Feature_Domain::SHARED_SCALAR => {
                Self::get_shared_scalar(intent_config, input, &feature_type, arguments)
            }
            Feature_Domain::SHARED_VECTOR => {
                Self::get_shared_vector(intent_config, input, &feature_type, arguments)
            }
        }
    }

    fn get_shared_scalar(intent_config: &IntentConfig,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Type,
                         arguments: &[Feature_Argument])
                         -> Result<Vec<f32>> {
        Ok(match *feature_type {
            Feature_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = intent_config.get_gazetteer(arguments[0].get_gazetteer())?;
                ::features::shared_scalar::has_gazetteer_hits(input, gazetteer)
            }
            Feature_Type::NGRAM_MATCHER => {
                ::features::shared_scalar::ngram_matcher(input, arguments[0].get_str())
            }
            Feature_Type::GET_MESSAGE_LENGTH => {
                ::features::shared_scalar::get_message_length(input,
                                                              arguments[0].get_scalar() as f32)
            }
            feature_type => panic!("Feature function not implemented: {:?}", feature_type),
        })
    }

    fn get_shared_vector(intent_config: &IntentConfig,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Type,
                         arguments: &[Feature_Argument])
                         -> Result<Vec<f32>> {
        Ok(match *feature_type {
            Feature_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = intent_config.get_gazetteer(arguments[0].get_gazetteer())?;
                ::features::shared_vector::has_gazetteer_hits(input, gazetteer)
            }
            Feature_Type::NGRAM_MATCHER => {
                ::features::shared_vector::ngram_matcher(input, arguments[0].get_str())
            }
            Feature_Type::IS_CAPITALIZED => ::features::shared_vector::is_capitalized(input),
            Feature_Type::IS_FIRST_WORD => ::features::shared_vector::is_first_word(input),
            Feature_Type::IS_LAST_WORD => ::features::shared_vector::is_last_word(input),
            Feature_Type::CONTAINS_POSSESSIVE => {
                ::features::shared_vector::contains_possessive(input)
            }
            Feature_Type::CONTAINS_DIGITS => ::features::shared_vector::contains_digits(input),
            feature_type => panic!("Feature functions not implemented: {:?}", feature_type),
        })
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;

    use protobuf;

    use protos::model::Model;
    use preprocessing::preprocess;
    use FileConfiguration;
    use file_path;
    use testutils::parse_json;
    use testutils::create_transposed_array;
    use super::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};

    #[derive(Deserialize)]
    struct TestDescription {
        text: String,
        //output: Vec<Vec<f32>>,
        features: Vec<Vec<f32>>,
    }

    #[test]
    #[ignore]
    // QKFIX: Temporarily ignore this test, waiting for update of protobufs
    fn feature_processor_works() {
        let file_configuration = FileConfiguration::default();
        let paths =
            fs::read_dir(file_path("snips-sdk-models-protobuf/tests/intent_classification/"))
                .unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_path = file_configuration.intent_classifier_path(path.file_stem().unwrap().to_str().unwrap());
            let mut model_file = File::open(model_path).unwrap();
            let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();

            for test in tests {
                let preprocessor_result = preprocess(&test.text);
                let feature_processor = ProtobufMatrixFeatureProcessor::new(&file_configuration,
                                                                            &model.get_features());

                let result = feature_processor.compute_features(&preprocessor_result);
                assert_eq!(result,
                           create_transposed_array(&test.features),
                           "for {:?}, input: {}",
                           path.file_stem().unwrap(),
                           &test.text);
            }
        }
    }
}
