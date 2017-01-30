use preprocessing::PreprocessorResult;
use models::gazetteer::{Gazetteer, HashSetGazetteer};
use models::model::Feature;
use models::features::{has_gazetteer_hits, ngram_matcher};

type FeatureFunction = Fn(&PreprocessorResult) -> Vec<f64>;

pub trait VectorFeatureProcessor {
    fn compute_features(&self, input: &PreprocessorResult) -> Vec<f64>;
}

pub struct ProtobufVectorFeatureProcessor<'a> {
    feature_functions: &'a [Feature],
}

impl<'a> ProtobufVectorFeatureProcessor<'a> {
    fn new(features: &'a [Feature]) -> ProtobufVectorFeatureProcessor<'a> {
        ProtobufVectorFeatureProcessor { feature_functions: features }
    }
}

impl Feature {
    fn compute(&self, input: &PreprocessorResult) -> Vec<f64> {
        match &*self.function_name {
            "hasGazetteerHits" => {
                has_gazetteer_hits(input,
                                   HashSetGazetteer::new(self.get_arguments()[0].get_gazetteer())
                                       .unwrap())
            }
            "ngramMatcher" => ngram_matcher(input, self.get_arguments()[0].get_str()),
            _ => panic!("method name not found"),
        }
    }
}

impl<'a> VectorFeatureProcessor for ProtobufVectorFeatureProcessor<'a> {
    fn compute_features(&self, input: &PreprocessorResult) -> Vec<f64> {
        return self.feature_functions
            .iter()
            .flat_map(|a_feature_function| a_feature_function.compute(input))
            .collect();
    }
}

#[cfg(test)]
mod test {
    extern crate protobuf;

    use std::fs::File;
    use std::path::Path;
    use models::model::Model;
    use super::{VectorFeatureProcessor, ProtobufVectorFeatureProcessor};
    use preprocessing::preprocess;

    #[test]
    // TODO: perform end-to-end tests
    fn feature_processor_works() {
        let file_path = "../data/snips-sdk-models/protos/output/BookRestaurant.pbbin";
        let mut is = File::open(&Path::new(file_path)).unwrap();
        let model = protobuf::parse_from_reader::<Model>(&mut is).unwrap();

        let preprocess_result = preprocess("Book me a restaurant to home");
        let feature_processor = ProtobufVectorFeatureProcessor::new(&model.get_features());

        let result = feature_processor.compute_features(&preprocess_result);
        println!("{:?}", result)
    }
}
