use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;

use pipeline::FeatureProcessor;
use pipeline::probabilistic::configuration::Feature;
use utils::token::Token;
use serde_json;

use errors::*;
use super::features;
use super::crf_utils::TaggingScheme;
use models::gazetteer::{HashSetGazetteer, StaticMapGazetteer};
use models::stemmer::StaticMapStemmer;
use models::word_clusterer::StaticMapWordClusterer;
use builtin_entities::{BuiltinEntityKind, RustlingParser};
use rustling_ontology::Lang;

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
        "is_digit" => is_digit_feature_function(offsets),
        "is_first" => is_first_feature_function(offsets),
        "is_last" => is_last_feature_function(offsets),
        "get_ngram_fn" => ngram_feature_function(&f.args, offsets),
        "get_shape_ngram_fn" => shape_ngram_feature_function(&f.args, offsets),
        "get_prefix_fn" => prefix_feature_function(&f.args, offsets),
        "get_suffix_fn" => suffix_feature_function(&f.args, offsets),
        "get_token_is_in_fn" => token_is_in_feature_function(&f.args, offsets),
        "get_is_in_gazetteer_fn" => is_in_gazetteer_feature_function(&f.args, offsets),
        "get_word_cluster_fn" => word_cluster_feature_function(&f.args, offsets),
        "get_built_in_annotation_fn" => builtin_entities_annotation_feature_function(&f.args, offsets),
        _ => bail!("Feature {} not implemented", f.factory_name),
    }
}

fn is_digit_feature_function(offsets: Vec<i32>) -> Result<FeatureFunction> {
    Ok(FeatureFunction::new("is_digit".to_string(), offsets, |t, i| features::is_digit(&t[i].value)))
}

fn is_first_feature_function(offsets: Vec<i32>) -> Result<FeatureFunction> {
    Ok(FeatureFunction::new("is_first".to_string(), offsets, |_, i| features::is_first(i)))
}

fn is_last_feature_function(offsets: Vec<i32>) -> Result<FeatureFunction> {
    Ok(FeatureFunction::new("is_last".to_string(), offsets, |t, i| features::is_last(t, i)))
}

fn ngram_feature_function(args: &HashMap<String, serde_json::Value>,
                          offsets: Vec<i32>) -> Result<FeatureFunction> {
    let n = parse_as_u64(args, "n")? as usize;
    let language_code = parse_as_string(args, "language_code")?;
    let common_words_gazetteer_name = parse_as_opt_string(args, "common_words_gazetteer_name")?;
    let use_stemming = parse_as_bool(args, "use_stemming")?;
    let common_words_gazetteer = if let Some(name) = common_words_gazetteer_name {
        Some(StaticMapGazetteer::new(&name, &language_code, use_stemming)?)
    } else {
        None
    };
    let stemmer = get_stemmer(language_code, use_stemming);
    Ok(FeatureFunction::new(
        format!("ngram_{}", n),
        offsets,
        move |tokens, token_index|
            features::ngram(tokens, token_index, n, stemmer.as_ref(), common_words_gazetteer.as_ref())))
}

fn shape_ngram_feature_function(args: &HashMap<String, serde_json::Value>,
                                offsets: Vec<i32>) -> Result<FeatureFunction> {
    let n = parse_as_u64(args, "n")? as usize;
    Ok(FeatureFunction::new(format!("shape_ngram_{}", n),
                            offsets,
                            move |t, i| features::shape(t, i, n)))
}

fn prefix_feature_function(args: &HashMap<String, serde_json::Value>,
                           offsets: Vec<i32>) -> Result<FeatureFunction> {
    let n = parse_as_u64(args, "prefix_size")? as usize;
    Ok(FeatureFunction::new(format!("prefix-{}", n),
                            offsets,
                            move |t, i| features::prefix(&t[i].value, n)))
}

fn suffix_feature_function(args: &HashMap<String, serde_json::Value>,
                           offsets: Vec<i32>) -> Result<FeatureFunction> {
    let n = parse_as_u64(args, "suffix_size")? as usize;
    Ok(FeatureFunction::new(format!("suffix-{}", n),
                            offsets,
                            move |t, i| features::suffix(&t[i].value, n)))
}

fn token_is_in_feature_function(args: &HashMap<String, serde_json::Value>,
                                offsets: Vec<i32>) -> Result<FeatureFunction> {
    let tokens_collection = parse_as_vec_string(args, "tokens_collection")?;
    let collection_name = parse_as_string(args, "collection_name")?;
    let language_code = parse_as_string(args, "language_code")?;
    let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
    let use_stemming = parse_as_bool(args, "use_stemming")?;
    let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
    let tokens_gazetteer = HashSetGazetteer::from(tokens_collection.into_iter());
    let stemmer = get_stemmer(language_code, use_stemming);
    Ok(FeatureFunction::new(
        format!("token_is_in_{}", collection_name),
        offsets,
        move |tokens, token_index|
            features::is_in_gazetteer(tokens,
                                      token_index,
                                      &tokens_gazetteer,
                                      stemmer.as_ref(),
                                      tagging_scheme)))
}

fn is_in_gazetteer_feature_function(args: &HashMap<String, serde_json::Value>,
                                    offsets: Vec<i32>) -> Result<FeatureFunction> {
    let gazetteer_name = parse_as_string(args, "gazetteer_name")?;
    let language_code = parse_as_string(args, "language_code")?;
    let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
    let use_stemming = parse_as_bool(args, "use_stemming")?;
    let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
    let gazetteer = StaticMapGazetteer::new(&gazetteer_name, &language_code, use_stemming)?;
    let stemmer = get_stemmer(language_code, use_stemming);
    Ok(FeatureFunction::new(
        format!("is_in_gazetteer_{}", gazetteer_name),
        offsets,
        move |tokens, token_index|
            features::is_in_gazetteer(tokens,
                                      token_index,
                                      &gazetteer,
                                      stemmer.as_ref(),
                                      tagging_scheme)))
}

fn word_cluster_feature_function(args: &HashMap<String, serde_json::Value>,
                                 offsets: Vec<i32>) -> Result<FeatureFunction> {
    let cluster_name = parse_as_string(args, "cluster_name")?;
    let language_code = parse_as_string(args, "language_code")?;
    let use_stemming = parse_as_bool(args, "use_stemming")?;
    let word_clusterer = StaticMapWordClusterer::new(language_code.clone(),
                                                     cluster_name.clone())?;
    let stemmer = get_stemmer(language_code, use_stemming);
    Ok(FeatureFunction::new(
        format!("word_cluster_{}", cluster_name),
        offsets,
        move |tokens, token_index|
            features::get_word_cluster(tokens,
                                       token_index,
                                       &word_clusterer,
                                       stemmer.as_ref())))
}

fn builtin_entities_annotation_feature_function(args: &HashMap<String, serde_json::Value>,
                                                offsets: Vec<i32>) -> Result<FeatureFunction> {
    let builtin_entity_label = parse_as_string(args, "built_in_entity_label")?;
    let builtin_entity_kind = BuiltinEntityKind::from_identifier(&builtin_entity_label).ok();
    let language_code = parse_as_string(args, "language_code")?;
    let builtin_parser = Lang::from_str(&language_code).ok()
        .map(|rust_lang| RustlingParser::get(rust_lang));
    let tagging_scheme_code = parse_as_u64(args, "tagging_scheme_code")? as u8;
    let tagging_scheme = TaggingScheme::from_u8(tagging_scheme_code)?;
    Ok(FeatureFunction::new(
        format!("built-in-{}", &builtin_entity_label),
        offsets,
        move |tokens, token_index|
            if let (Some(parser), Some(builtin_entity_kind)) = (builtin_parser.as_ref(), builtin_entity_kind) {
                features::get_builtin_entities_annotation(
                    tokens,
                    token_index,
                    &**parser,
                    builtin_entity_kind,
                    tagging_scheme)
            } else {
                None
            }
    ))
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

fn parse_as_vec_string(args: &HashMap<String, serde_json::Value>, arg_name: &str) -> Result<Vec<String>> {
    args.get(arg_name)
        .ok_or(format!("can't retrieve '{}' parameter", arg_name))?
        .as_array()
        .ok_or(format!("'{}' isn't an array", arg_name))?
        .iter()
        .map(|v|
            Ok(v.as_str()
                .ok_or(format!("'{}' is not a string", v))?
                .to_string())
        )
        .collect()
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

fn get_stemmer(language_code: String, use_stemming: bool) -> Option<StaticMapStemmer> {
    if use_stemming {
        StaticMapStemmer::new(language_code).ok()
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use pipeline::FeatureProcessor;

    use utils::token::tokenize;

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
