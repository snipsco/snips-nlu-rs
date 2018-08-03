use itertools::Itertools;
use std::collections::HashMap;

use models::FeatureFactory;
use errors::*;
use nlu_utils::token::Token;
use resources::SharedResources;
use std::sync::Arc;
use slot_filler::features::*;

pub struct ProbabilisticFeatureProcessor {
    features: Vec<Box<Feature>>,
}

impl ProbabilisticFeatureProcessor {
    // TODO add a `GazetteerProvider` to this signature
    pub fn new(
        features: &[FeatureFactory],
        shared_resources: Arc<SharedResources>,
    ) -> Result<ProbabilisticFeatureProcessor> {
        let features = features
            .iter()
            .map(|f| get_features(f, shared_resources.clone()))
            .collect::<Result<Vec<Vec<_>>>>()?
            .into_iter()
            .flat_map(|fs| fs)
            .collect();

        Ok(ProbabilisticFeatureProcessor { features })
    }
}

impl ProbabilisticFeatureProcessor {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn compute_features(&self, input: &&[Token]) -> Vec<Vec<(String, String)>> {
        self.features
            .iter()
            .fold(vec![vec![]; input.len()], |mut acc, f| {
                (0..input.len()).foreach(|i| {
                    if let Some(value) = f.compute(input, i) {
                        f.offsets_with_name().iter().foreach(|&(offset, ref key)| {
                            if i as i32 - offset >= 0 && i as i32 - offset < input.len() as i32 {
                                acc[(i as i32 - offset) as usize].push(
                                    (key.clone(), value.clone())
                                );
                            }
                        });
                    }
                });
                acc
            })
    }
}

pub trait Feature: Send + Sync {
    //    function: Box<Fn(&[Token], usize) -> Option<String> + Send + Sync>,
//    offsets: Vec<(i32, String)>,
    fn base_name(&self) -> &'static str;
    fn name(&self) -> String { self.base_name().to_string() }
    fn offsets(&self) -> &[i32];
    fn build_features(
        offsets: &[i32],
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> where Self: Sized;
    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String>;

    fn offsets_with_name(&self) -> Vec<(i32, String)> {
        self.offsets()
            .iter()
            .map(|i| {
                (
                    *i,
                    if *i == 0 {
                        self.name().to_string()
                    } else {
                        format!("{}[{:+}]", self.name(), i)
                    },
                )
            })
            .collect()
    }
}

fn get_features(
    f: &FeatureFactory,
    shared_resources: Arc<SharedResources>,
) -> Result<Vec<Box<Feature>>> {
    let offsets = f.offsets.clone();
    match f.factory_name.as_ref() {
        "is_digit" => IsDigitFeature::build_features(&offsets, &f.args, shared_resources),
        "length" => LengthFeature::build_features(&offsets, &f.args, shared_resources),
        "is_first" => IsFirstFeature::build_features(&offsets, &f.args, shared_resources),
        "is_last" => IsLastFeature::build_features(&offsets, &f.args, shared_resources),
        "ngram" => NgramFeature::build_features(&offsets, &f.args, shared_resources),
        "shape_ngram" => ShapeNgramFeature::build_features(&offsets, &f.args, shared_resources),
        "prefix" => PrefixFeature::build_features(&offsets, &f.args, shared_resources),
        "suffix" => SuffixFeature::build_features(&offsets, &f.args, shared_resources),
        "entity_match" => EntityMatchFeature::build_features(&offsets, &f.args, shared_resources),
        "builtin_entity_match" => BuiltinEntityMatchFeature::build_features(&offsets, &f.args, shared_resources),
        "word_cluster" => WordClusterFeature::build_features(&offsets, &f.args, shared_resources),
        _ => bail!("Feature {} not implemented", f.factory_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nlu_utils::language::Language;
    use nlu_utils::token::tokenize;

    #[test]
    fn compute_features_works() {
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features: vec![
                Feature::new("Toto", vec![0], |_, i| {
                    if i == 0 {
                        None
                    } else {
                        Some("Foobar".to_string())
                    }
                }),
            ],
        };

        let computed_features =
            fp.compute_features(&tokenize("hello world how are you ?", language).as_slice());

        assert_eq!(computed_features.len(), 6);
        assert_eq!(computed_features[0], vec![]);
        for i in 1..5 {
            assert_eq!(
                computed_features[i],
                vec![("Toto".to_string(), "Foobar".to_string())]
            );
        }
    }

    #[test]
    fn offset_works() {
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features: vec![
                Feature::new("Toto", vec![-2, 0, 2, 4], |x, i| {
                    if i == 0 {
                        None
                    } else {
                        Some(x[i].value.clone())
                    }
                }),
                Feature::new("Tutu", vec![2], |_, i| {
                    if i != 3 {
                        None
                    } else {
                        Some("Foobar".to_string())
                    }
                }),
            ],
        };

        let computed_features =
            fp.compute_features(&tokenize("hello world how are you ?", language).as_slice());
        assert_eq!(
            computed_features,
            vec![
                vec![
                    ("Toto[+2]".to_string(), "how".to_string()),
                    ("Toto[+4]".to_string(), "you".to_string()),
                ],
                vec![
                    ("Toto".to_string(), "world".to_string()),
                    ("Toto[+2]".to_string(), "are".to_string()),
                    ("Toto[+4]".to_string(), "?".to_string()),
                    ("Tutu[+2]".to_string(), "Foobar".to_string()),
                ],
                vec![
                    ("Toto".to_string(), "how".to_string()),
                    ("Toto[+2]".to_string(), "you".to_string()),
                ],
                vec![
                    ("Toto[-2]".to_string(), "world".to_string()),
                    ("Toto".to_string(), "are".to_string()),
                    ("Toto[+2]".to_string(), "?".to_string()),
                ],
                vec![
                    ("Toto[-2]".to_string(), "how".to_string()),
                    ("Toto".to_string(), "you".to_string()),
                ],
                vec![
                    ("Toto[-2]".to_string(), "are".to_string()),
                    ("Toto".to_string(), "?".to_string()),
                ],
            ]
        );
    }
}
