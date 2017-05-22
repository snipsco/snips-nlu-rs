use itertools::Itertools;

use pipeline::FeatureProcessor;
use pipeline::probabilistic::configuration::Feature;
use preprocessing::Token;

use errors::*;
use super::features;

struct FeatureFunction {
    function: Box<Fn(&[Token], usize) -> Option<String>>,
    offsets: Vec<(i32, String)>,
}

impl FeatureFunction {
    fn new<T: 'static>(key: String, offsets: Vec<i32>, function: T) -> FeatureFunction
        where T: Fn(&[Token], usize) -> Option<String>
    {
        let offsets = offsets
            .into_iter()
            .map(|i| {
                     (i,
                      if i == 0 {
                          key.clone()
                      } else {
                          format!("{}[{:+}]", key, i)
                      })
                 })
            .collect();
        let function = Box::new(function);
        FeatureFunction { offsets, function }
    }
}

pub struct ProbabilisticFeatureProcessor {
    functions: Vec<FeatureFunction>,
}

impl FeatureProcessor<Vec<Token>, Vec<Vec<(String, String)>>> for ProbabilisticFeatureProcessor {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn compute_features(&self, input: &Vec<Token>) -> Vec<Vec<(String, String)>> {
        self.functions
            .iter()
            .fold(vec![vec![]; input.len()], |mut acc, f| {
                (0..input.len()).foreach(|i| {
                    if let Some(value) = (f.function)(input, i) {
                        f.offsets.iter().foreach(|&(offset, ref key)| {
                            if i as i32 - offset >= 0 && i as i32 - offset < input.len() as i32 {
                                acc[(i as i32 - offset) as usize].push((key.clone(), value.clone()));
                            }
                        });
                    }
                });
                acc
            })
    }
}

impl ProbabilisticFeatureProcessor {
    // TODO add a `GazetteerProvider` to this signature
    pub fn new(features: &[Feature]) -> Result<ProbabilisticFeatureProcessor> {
        let functions: Result<Vec<FeatureFunction>> =
            features.iter().map(|f| get_feature_function(f)).collect();

        Ok(ProbabilisticFeatureProcessor { functions: functions? })
    }
}

fn get_feature_function(f: &Feature) -> Result<FeatureFunction> {
    let offsets = f.offsets.clone();
    match &*f.factory_name {
        // TODO use proper type from protobuf
        "is_first" => {
            Ok(FeatureFunction::new("is_first".to_string(),
                                    offsets,
                                    |_, i| features::is_first(i)))
        }
        "is_last" => {
            Ok(FeatureFunction::new("is_last".to_string(),
                                    offsets,
                                    |t, i| features::is_last(t, i)))
        }
        "get_ngram_fn" => {
            let n = f.args
                .get("n")
                .ok_or("can't retrieve 'n' parameter")?
                .as_u64()
                .ok_or("'n' isn't an u64")? as usize;
            Ok(FeatureFunction::new(format!("ngram_{}", n),
                                    offsets,
                                    move |t, i| features::ngram(t, i, n)))
        }
        _ => bail!("Feature {} not implemented", f.factory_name),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use pipeline::FeatureProcessor;

    use preprocessing::tokenize;

    #[test]
    fn compute_features_works() {
        let fp = ProbabilisticFeatureProcessor {
            functions: vec![FeatureFunction::new("Toto".to_string(), vec![0], |_, i| if i == 0 {
                None
            } else {
                Some("Foobar".to_string())
            })],
        };

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?"));


        assert_eq!(computed_features.len(), 5);
        assert_eq!(computed_features[0], vec![]);
        for i in 1..5 {
            assert_eq!(computed_features[i],
                       vec![("Toto".to_string(), "Foobar".to_string())]);
        }
    }

    #[test]
    fn offset_works() {
        let fp = ProbabilisticFeatureProcessor {
            functions: vec![FeatureFunction::new("Toto".to_string(),
                                                 vec![-2, 0, 2, 4],
                                                 |x, i| if i == 0 {
                                                     None
                                                 } else {
                                                     Some(x[i].value.clone())
                                                 }),
                            FeatureFunction::new("Tutu".to_string(), vec![2], |_, i| if i != 3 {
                None
            } else {
                Some("Foobar".to_string())
            })],
        };

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?"));

        assert_eq!(computed_features,
                   vec![vec![("Toto[+2]".to_string(), "how".to_string()),
                             ("Toto[+4]".to_string(), "you".to_string())],
                        vec![("Toto".to_string(), "world".to_string()),
                             ("Toto[+2]".to_string(), "are".to_string()),
                             ("Tutu[+2]".to_string(), "Foobar".to_string())],
                        vec![("Toto".to_string(), "how".to_string()),
                             ("Toto[+2]".to_string(), "you".to_string())],
                        vec![("Toto[-2]".to_string(), "world".to_string()),
                             ("Toto".to_string(), "are".to_string())],
                        vec![("Toto[-2]".to_string(), "how".to_string()),
                             ("Toto".to_string(), "you".to_string())]]);
    }
}
