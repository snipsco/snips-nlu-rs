use std::rc::Rc;

use config::ArcBoxedIntentConfig;
use itertools::Itertools;
use pipeline::FeatureProcessor;
use preprocessing::Token;
use protos::feature::{Feature, Feature_Vector_Type};

use super::features;

struct FeatureFunction {
    function: Box<Fn(&Vec<Token>, usize) -> Option<(String, String)>>,
    offsets: Option<Vec<i32>>
}

struct CrfFeatureProcessor {
    functions: Vec<FeatureFunction>
}

impl FeatureProcessor<Vec<Token>, Vec<Vec<(String, String)>>> for CrfFeatureProcessor {
    fn compute_features(&self, input: &Vec<Token>) -> Vec<Vec<(String, String)>> {
        self.functions.iter().fold(vec![vec![]; input.len()], |mut acc, f| {
            (0..input.len()).foreach(|i| {
                if let Some(kv) = (f.function)(input, i) {
                    if let Some(ref offsets) = f.offsets {
                        offsets.iter().foreach(|offset| {
                            if *offset != 0
                                && i as i32 - offset >= 0
                                && i as i32 - offset < input.len() as i32 {
                                acc[(i as i32 - offset) as usize].push(offset_key(&kv, *offset));
                            }
                        })
                    }
                    acc[i].push(kv)
                }
            });
            acc
        })
    }
}

fn offset_key(&(ref key, ref value): &(String, String), offset: i32) -> (String, String) {
    (format!("{}[{:+}]", key, offset), value.clone())
}

impl CrfFeatureProcessor {
    fn new(config: ArcBoxedIntentConfig, features: &[Feature]) -> CrfFeatureProcessor {
        let functions = features.iter().map(|f| get_feature_function(f)).collect();

        CrfFeatureProcessor { functions }
    }
}

fn get_feature_function(f: &Feature) -> FeatureFunction {
    let function: Box<Fn(&Vec<Token>, usize) -> Option<(String, String)>> =
        match f.get_vector_type() {
            // TODO use proper type from protobuf
            Feature_Vector_Type::IS_FIRST_WORD => Box::new(|_, i| features::is_first(i)),
            Feature_Vector_Type::IS_LAST_WORD => Box::new(|t, i| features::is_last(t, i)), // TODO feature key should not be computed each time
            _ => panic!()
        };

    let offsets = None; // TODO get that from protobuf when added

    FeatureFunction { function, offsets }
}


#[cfg(test)]
mod tests {
    use super::*;

    use pipeline::FeatureProcessor;

    use preprocessing::light::tokenize;

    #[test]
    fn compute_features_works() {
        let fp = CrfFeatureProcessor {
            functions: vec![FeatureFunction {
                function: Box::new(|_, i|
                    if i == 0 { None } else {
                        Some(("Toto".to_string(), "Foobar".to_string()))
                    }
                ),
                offsets: None
            }]
        };

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?"));


        assert_eq!(computed_features.len(), 5);
        assert_eq!(computed_features[0], vec![]);
        for i in 1..5 {
            assert_eq!(computed_features[i], vec![("Toto".to_string(), "Foobar".to_string())]);
        }
    }

    #[test]
    fn offset_works() {
        let fp = CrfFeatureProcessor {
            functions: vec![FeatureFunction {
                function: Box::new(|x, i|
                    if i == 0 { None } else {
                        Some(("Toto".to_string(), x[i].value.clone()))
                    }
                ),
                offsets: Some(vec![-2, 0, 2, 4])
            }]
        };

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?"));


        assert_eq!(computed_features, vec![
            vec![("Toto[+2]".to_string(), "how".to_string()),
                 ("Toto[+4]".to_string(), "you".to_string())],
            vec![("Toto".to_string(), "world".to_string()),
                 ("Toto[+2]".to_string(), "are".to_string()), ],
            vec![("Toto".to_string(), "how".to_string()),
                 ("Toto[+2]".to_string(), "you".to_string()), ],
            vec![("Toto[-2]".to_string(), "world".to_string()),
                 ("Toto".to_string(), "are".to_string()), ],
            vec![("Toto[-2]".to_string(), "how".to_string()),
                 ("Toto".to_string(), "you".to_string()), ],
        ]);
    }
}






