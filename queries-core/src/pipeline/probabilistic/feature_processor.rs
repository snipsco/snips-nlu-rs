use itertools::Itertools;
use std::collections::HashMap;

use pipeline::FeatureProcessor;
use pipeline::probabilistic::configuration::Feature;
use preprocessing::Token;
use serde_json;

use errors::*;
use super::features;
use super::crf_utils::TaggingScheme;
use models::gazetteer::{HashSetGazetteer, StaticMapGazetteer};
use models::stemmer::StaticMapStemmer;

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
            let n = parse_as_u64(&f.args, "n")? as usize;
            let language_code = parse_as_string(&f.args, "language_code")?;
            let common_words_gazetteer_name = parse_as_opt_string(&f.args, "common_words_gazetteer_name")?;
            let use_stemming = parse_as_bool(&f.args, "use_stemming")?;
            let common_words_gazetteer = if let Some(name) = common_words_gazetteer_name {
                Some(StaticMapGazetteer::new(&name, &language_code, use_stemming)?)
            } else {
                None
            };
            let stemmer = if use_stemming {
                Some(StaticMapStemmer::new(language_code.to_string())?)
            } else {
                None
            };

            Ok(FeatureFunction::new(
                format!("ngram_{}", n),
                offsets,
                move |tokens, token_index|
                    features::ngram(tokens, token_index, n, stemmer.as_ref(), common_words_gazetteer.as_ref())))
        }
        "get_shape_ngram_fn" => {
            let n = parse_as_u64(&f.args, "n")? as usize;
            Ok(FeatureFunction::new(format!("shape_ngram_{}", n),
                                    offsets,
                                    move |t, i| features::shape(t, i, n)))
        }
        "get_prefix_fn" => {
            let n = parse_as_u64(&f.args, "n")? as usize;
            Ok(FeatureFunction::new(format!("prefix-{}", n),
                                    offsets,
                                    move |t, i| features::prefix(&t[i].value, n)))
        }
        "get_suffix_fn" => {
            let n = parse_as_u64(&f.args, "n")? as usize;
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

            let collection_name = parse_as_string(&f.args, "collection_name")?;
            let language_code = parse_as_string(&f.args, "language_code")?;
            let tagging_scheme_code = parse_as_u64(&f.args, "tagging_scheme_code")? as u8;
            let use_stemming = parse_as_bool(&f.args, "use_stemming")?;

            let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
            let tokens_gazetteer = HashSetGazetteer::from(tokens_collection.into_iter());
            let stemmer = if use_stemming {
                Some(StaticMapStemmer::new(language_code.to_string())?)
            } else {
                None
            };

            Ok(FeatureFunction::new(
                format!("token_is_in_{}", collection_name),
                offsets,
                move |tokens, token_index|
                    features::is_in_gazetteer(tokens,
                                              token_index,
                                              &tokens_gazetteer,
                                              stemmer.as_ref(),
                                              &tagging_scheme)))
        }
        "get_is_in_gazetteer_fn" => {
            let gazetteer_name = parse_as_string(&f.args, "gazetteer_name")?;
            let language_code = parse_as_string(&f.args, "language_code")?;
            let tagging_scheme_code = parse_as_u64(&f.args, "tagging_scheme_code")? as u8;
            let use_stemming = parse_as_bool(&f.args, "use_stemming")?;
            let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
            let gazetteer = StaticMapGazetteer::new(&gazetteer_name, &language_code, use_stemming)?;
            let stemmer = if use_stemming {
                Some(StaticMapStemmer::new(language_code.to_string())?)
            } else {
                None
            };

            Ok(FeatureFunction::new(
                format!("is_in_gazetteer_{}", gazetteer_name),
                offsets,
                move |tokens, token_index|
                    features::is_in_gazetteer(tokens,
                                              token_index,
                                              &gazetteer,
                                              stemmer.as_ref(),
                                              &tagging_scheme)))
        }
        "get_word_cluster_fn" => {
            let cluster_name = parse_as_string(&f.args, "cluster_name")?;
            let language_code = parse_as_string(&f.args, "language_code")?;

            Ok(FeatureFunction::new(
                format!("word_cluster_{}", cluster_name),
                offsets,
                move |tokens, token_index|
                    features::get_word_cluster(tokens,
                                               token_index,
                                               &cluster_name,
                                               &language_code)))
        }
        _ => bail!("Feature {} not implemented", f.factory_name),
    }
}

fn parse_as_string(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<String> {
    Ok(args.get(arg_name)
        .ok_or(format!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .ok_or(format!("'{}' isn't a string", arg_name))?
        .to_string()
    )
}

fn parse_as_opt_string(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<Option<String>> {
    Ok(args.get(arg_name)
        .ok_or(format!("can't retrieve '{}' parameter", arg_name))?
        .as_str()
        .map(|s| s.to_string())
    )
}

fn parse_as_bool(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<bool> {
    Ok(args.get(arg_name)
        .ok_or(format!("can't retrieve '{}' parameter", arg_name))?
        .as_bool()
        .ok_or(format!("'{}' isn't a bool", arg_name))?
    )
}

fn parse_as_u64(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<u64> {
    Ok(args.get(arg_name)
        .ok_or(format!("can't retrieve '{}' parameter", arg_name))?
        .as_u64()
        .ok_or(format!("'{}' isn't a u64", arg_name))?
    )
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
