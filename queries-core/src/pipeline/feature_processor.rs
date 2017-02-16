use errors::*;
use ndarray::{ Array, Array2 };

use ::FileConfiguration;
use preprocessing::PreprocessorResult;
use models::gazetteer::HashSetGazetteer;
use models::model::{ Feature, Feature_Type, Feature_Domain, Argument };

pub trait MatrixFeatureProcessor {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f64>;
}

pub struct ProtobufMatrixFeatureProcessor<'a> {
    file_configuration: &'a FileConfiguration,
    feature_functions: &'a [Feature],
}

impl<'a> ProtobufMatrixFeatureProcessor<'a> {
    pub fn new(file_configuration: &'a FileConfiguration, features: &'a [Feature]) -> ProtobufMatrixFeatureProcessor<'a> {
        ProtobufMatrixFeatureProcessor {
            file_configuration: file_configuration,
            feature_functions: features
        }
    }
}

impl<'a> MatrixFeatureProcessor for ProtobufMatrixFeatureProcessor<'a> {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f64> {
        let computed_values = self.feature_functions
            .iter()
            .flat_map(|feature_function| feature_function.compute(self.file_configuration, input).unwrap()) // TODO: Dunno how to kill this unwrap
            .collect::<Vec<f64>>();

        let len = self.feature_functions.len();
        let feature_length = computed_values.len() / len;

        match Array::from_vec(computed_values).into_shape((len, feature_length)) {
            Ok(array) => array,
            Err(_) => panic!("A feature function doesn't have the same len as the others."),
        }
    }
}

impl Feature {
    fn compute(&self, file_configuration: &FileConfiguration, input: &PreprocessorResult) -> Result<Vec<f64>> {
        let known_domain = self.get_known_domain();
        let feature_type = self.field_type;
        let arguments = self.get_arguments();

        match known_domain {
            Feature_Domain::SHARED_SCALAR => {
                Self::get_shared_scalar(file_configuration, input, &feature_type, arguments)
            }
            Feature_Domain::SHARED_VECTOR => {
                Self::get_shared_vector(file_configuration, input, &feature_type, arguments)
            }
        }
    }

    fn get_shared_scalar(file_configuration: &FileConfiguration,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Type,
                         arguments: &[Argument])
                         -> Result<Vec<f64>> {
        // TODO: this Result::Ok smells something bad
        Ok(match *feature_type {
            Feature_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = HashSetGazetteer::new(file_configuration, arguments[0].get_gazetteer())?;
                ::features::shared_scalar::has_gazetteer_hits(input, &gazetteer)
            }
            Feature_Type::NGRAM_MATCHER => {
                ::features::shared_scalar::ngram_matcher(input, arguments[0].get_str())
            }
            _ => panic!("Feature function not implemented")
        })
    }

    fn get_shared_vector(file_configuration: &FileConfiguration,
                         input: &PreprocessorResult,
                         feature_type: &Feature_Type,
                         arguments: &[Argument])
                         -> Result<Vec<f64>> {
        // TODO: Same as above
        Ok(match *feature_type {
            Feature_Type::HAS_GAZETTEER_HITS => {
                let gazetteer = HashSetGazetteer::new(file_configuration, arguments[0].get_gazetteer())?;
                ::features::shared_vector::has_gazetteer_hits(input, &gazetteer)
            }
            Feature_Type::NGRAM_MATCHER => {
                ::features::shared_vector::ngram_matcher(input, arguments[0].get_str())
            }
            _ => panic!("Feature functions not implemented")
        })
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;

    use protobuf;

    use models::model::Model;
    use preprocessing::preprocess;
    use FileConfiguration;
    use testutils::file_path;
    use testutils::parse_json;
    use testutils::create_transposed_array;
    use super::{MatrixFeatureProcessor, ProtobufMatrixFeatureProcessor};

    #[derive(Deserialize)]
    struct TestDescription {
        text: String,
        //output: Vec<Vec<f64>>,
        features: Vec<Vec<f64>>,
    }

    #[test]
    fn feature_processor_works() {
        let file_configuration = FileConfiguration::default();
        let paths = fs::read_dir(file_path("snips-sdk-models/tests/intent_classification/")).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_path = file_configuration.intent_classifier_path(path.file_stem().unwrap().to_str().unwrap());
            let mut model_file = File::open(model_path).unwrap();
            let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();

            for test in tests {
                let preprocessor_result = preprocess(&test.text);
                let feature_processor = ProtobufMatrixFeatureProcessor::new(&file_configuration, &model.get_features());

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
