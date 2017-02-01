use ndarray::{Array, Array2};
use preprocessing::PreprocessorResult;
use models::gazetteer::{Gazetteer, HashSetGazetteer};
use models::model::{Feature, Feature_Type};
use models::features::{has_gazetteer_hits, ngram_matcher};

pub trait MatrixFeatureProcessor {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f64>;
}

pub struct ProtobufMatrixFeatureProcessor<'a> {
    feature_functions: &'a [Feature],
}

impl<'a> ProtobufMatrixFeatureProcessor<'a> {
    pub fn new(features: &'a [Feature]) -> ProtobufMatrixFeatureProcessor<'a> {
        ProtobufMatrixFeatureProcessor { feature_functions: features }
    }
}

impl<'a> MatrixFeatureProcessor for ProtobufMatrixFeatureProcessor<'a> {
    fn compute_features(&self, input: &PreprocessorResult) -> Array2<f64> {
        let computed_values = self.feature_functions
            .iter()
            .flat_map(|feature_function| feature_function.compute(input))
            .collect::<Vec<f64>>();

        let len = self.feature_functions.len();
        let feature_length = computed_values.len() / len;

        match Array::from_vec(computed_values).into_shape((len, feature_length)) {
            Ok(array) => array,
            Err(_) => panic!("A feature function doesn't have the same len as the others.")
        }
    }
}

impl Feature {
    fn compute(&self, input: &PreprocessorResult) -> Vec<f64> {
        match self.field_type {
            Feature_Type::HAS_GAZETTEER_HITS => {
                has_gazetteer_hits(input,
                                   HashSetGazetteer::new(self.get_arguments()[0].get_gazetteer())
                                       .unwrap())
            }
            Feature_Type::NGRAM_MATCHER => ngram_matcher(input, self.get_arguments()[0].get_str()),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate protobuf;

    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use models::model::Model;
    use preprocessing::preprocess;
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
        let model_directory = "../data/snips-sdk-models-protobuf/intent_classification/";
        let paths = fs::read_dir("../data/snips-sdk-models/tests/intent_classification/").unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let tests: Vec<TestDescription> = parse_json(path.to_str().unwrap());

            let model_path = Path::new(model_directory)
                .join(path.file_stem().unwrap())
                .with_extension("pbbin");
            let mut model_file = File::open(model_path).unwrap();
            let model = protobuf::parse_from_reader::<Model>(&mut model_file).unwrap();

            for test in tests {
                let preprocess_result = preprocess(&test.text);
                let feature_processor = ProtobufMatrixFeatureProcessor::new(&model.get_features());
                let result = feature_processor.compute_features(&preprocess_result);
                assert_eq!(result, create_transposed_array(&test.features), "for {:?}, input: {}", path.file_stem().unwrap(), &test.text);
            }
        }
    }
}
