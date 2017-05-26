use itertools::Itertools;

use pipeline::FeatureProcessor;
use pipeline::probabilistic::configuration::Feature;
use preprocessing::Token;

use errors::*;
use super::features;
use super::crf_utils::TaggingScheme;

struct FeatureFunction {
    function: Box<Fn(&[Token], usize) -> Option<String> + Send + Sync>,
    offsets: Vec<(i32, String)>,
}

impl FeatureFunction {
    fn new<T>(key: String, offsets: Vec<i32>, function: T) -> FeatureFunction
        where T: Fn(&[Token], usize) -> Option<String> + Send + Sync + 'static
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
        FeatureFunction { offsets, function: Box::new(function) }
    }
}

pub struct ProbabilisticFeatureProcessor {
    functions: Vec<FeatureFunction>,
}

impl<'a> FeatureProcessor<&'a [Token], Vec<Vec<(String, String)>>> for ProbabilisticFeatureProcessor {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn compute_features(&self, input: &&'a [Token]) -> Vec<Vec<(String, String)>> {
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
        "is_digit" => {
            Ok(FeatureFunction::new("is_digit".to_string(),
                                    offsets,
                                    |t, i| features::is_digit(&t[i].value)))
        }
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
        "get_shape_ngram_fn" => {
            let n = f.args
                .get("n")
                .ok_or("can't retrieve 'n' parameter")?
                .as_u64()
                .ok_or("'n' isn't an u64")? as usize;
            Ok(FeatureFunction::new(format!("shape_ngram_{}", n),
                                    offsets,
                                    move |t, i| features::shape(t, i, n)))
        }
        "get_prefix_fn" => {
            let n = f.args
                .get("n")
                .ok_or("can't retrieve 'n' parameter")?
                .as_u64()
                .ok_or("'n' isn't an u64")? as usize;
            Ok(FeatureFunction::new(format!("prefix-{}", n),
                                    offsets,
                                    move |t, i| features::prefix(&t[i].value, n)))
        }
        "get_suffix_fn" => {
            let n = f.args
                .get("n")
                .ok_or("can't retrieve 'n' parameter")?
                .as_u64()
                .ok_or("'n' isn't an u64")? as usize;
            Ok(FeatureFunction::new(format!("suffix-{}", n),
                                    offsets,
                                    move |t, i| features::suffix(&t[i].value, n)))
        }
        "get_token_is_in_fn" => {
            let tokens_collection: Result<Vec<String>> = f.args
                .get("tokens_collection")
                .ok_or("can't retrieve 'tokens_collection' parameter")?
                .as_array()
                .ok_or("'tokens_collection' isn't an array")?
                .iter()
                .map(|v|
                    v.as_str()
                        .map(|s| s.to_string())
                        .ok_or(format!("'{}' is not a string", v).into()))
                .collect();

            let tokens_collection = tokens_collection?;

            let collection_name = f.args
                .get("collection_name")
                .ok_or("can't retrieve 'collection_name' parameter")?
                .as_str()
                .ok_or("'collection_name' isn't a string")?;
            let tagging_scheme_code = f.args
                .get("tagging_scheme_code")
                .ok_or("can't retrieve 'tagging_scheme_code' parameter")?
                .as_u64()
                .ok_or("'tagging_scheme_code' isn't a u64")? as u8;

            let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;

            Ok(FeatureFunction::new(
                format!("token_is_in_{}", collection_name),
                offsets,
                move |t, i| features::is_in_collection(t, i, &tokens_collection, &tagging_scheme)))
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

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?").as_slice());

        assert_eq!(computed_features.len(), 6);
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

        let computed_features = fp.compute_features(&tokenize("hello world how are you ?").as_slice());
        assert_eq!(computed_features,
        vec![vec![("Toto[+2]".to_string(), "how".to_string()),
                  ("Toto[+4]".to_string(), "you".to_string())],
             vec![("Toto".to_string(), "world".to_string()),
                  ("Toto[+2]".to_string(), "are".to_string()),
                  ("Toto[+4]".to_string(), "?".to_string()),
                  ("Tutu[+2]".to_string(), "Foobar".to_string())],
             vec![("Toto".to_string(), "how".to_string()),
                  ("Toto[+2]".to_string(), "you".to_string())],
             vec![("Toto[-2]".to_string(), "world".to_string()),
                  ("Toto".to_string(), "are".to_string()),
                  ("Toto[+2]".to_string(), "?".to_string())],
             vec![("Toto[-2]".to_string(), "how".to_string()),
                  ("Toto".to_string(), "you".to_string())],
             vec![("Toto[-2]".to_string(), "are".to_string()),
                  ("Toto".to_string(), "?".to_string())]]);
    }
}
