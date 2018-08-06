use itertools::Itertools;
use std::collections::HashMap;

use models::FeatureFactory;
use errors::*;
use nlu_utils::token::Token;
use resources::SharedResources;
use std::sync::Arc;
use slot_filler::features::*;

pub struct ProbabilisticFeatureProcessor {
    features_offsetters: Vec<FeatureOffsetter>,
}

impl ProbabilisticFeatureProcessor {
    pub fn new(
        features: &[FeatureFactory],
        shared_resources: Arc<SharedResources>,
    ) -> Result<ProbabilisticFeatureProcessor> {
        let features_offsetters = features
            .iter()
            .map(|f| get_features(f, shared_resources.clone()))
            .collect::<Result<Vec<Vec<_>>>>()?
            .into_iter()
            .flat_map(|fs| fs)
            .collect();

        Ok(ProbabilisticFeatureProcessor { features_offsetters })
    }
}

impl ProbabilisticFeatureProcessor {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn compute_features(&self, input: &&[Token]) -> Vec<Vec<(String, String)>> {
        self.features_offsetters
            .iter()
            .fold(vec![vec![]; input.len()], |mut acc, offsetter| {
                (0..input.len()).foreach(|i| {
                    if let Some(value) = offsetter.feature.compute(input, i) {
                        offsetter.offsets_with_name().iter().foreach(|&(offset, ref key)| {
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

struct FeatureOffsetter {
    feature: Box<Feature>,
    offsets: Vec<i32>
}

impl FeatureOffsetter {
    fn offsets_with_name(&self) -> Vec<(i32, String)> {
        self.offsets
            .iter()
            .map(|i| {
                (
                    *i,
                    if *i == 0 {
                        self.feature.name().to_string()
                    } else {
                        format!("{}[{:+}]", self.feature.name(), i)
                    },
                )
            })
            .collect()
    }
}

pub trait FeatureKindRepr {
    fn feature_kind(&self) -> FeatureKind;
}

pub trait Feature: FeatureKindRepr + Send + Sync {
    fn name(&self) -> String { self.feature_kind().identifier().to_string() }
    fn build_features(
        args: &HashMap<String, ::serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>> where Self: Sized;
    fn compute(&self, tokens: &[Token], token_index: usize) -> Option<String>;
}

get_features!([
    (IsDigitFeature, is_digit),
    (LengthFeature, length),
    (IsFirstFeature, is_first),
    (IsLastFeature, is_last),
    (NgramFeature, ngram),
    (ShapeNgramFeature, shape_ngram),
    (PrefixFeature, prefix),
    (SuffixFeature, suffix),
    (EntityMatchFeature, entity_match),
    (BuiltinEntityMatchFeature, builtin_entity_match),
    (WordClusterFeature, word_cluster)
]);

#[cfg(test)]
mod tests {
    use super::*;

    use nlu_utils::language::Language;
    use nlu_utils::token::tokenize;

    #[test]
    fn compute_features_works() {
        // Given
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features_offsetters: vec![
                FeatureOffsetter {
                    offsets: vec![0],
                    feature: Box::new(IsDigitFeature {}) as Box<_>
                },
                FeatureOffsetter {
                    offsets: vec![0],
                    feature: Box::new(LengthFeature {}) as Box<_>
                }
            ],
        };
        let tokens = tokenize("I prefer 7 over 777", language);

        // When
        let computed_features = fp.compute_features(&tokens.as_slice());

        let expected_features = vec![
            vec![("length".to_string(), "1".to_string())],
            vec![("length".to_string(), "6".to_string())],
            vec![("is_digit".to_string(), "1".to_string()), ("length".to_string(), "1".to_string())],
            vec![("length".to_string(), "4".to_string())],
            vec![("is_digit".to_string(), "1".to_string()), ("length".to_string(), "3".to_string())],
        ];

        // Then
        assert_eq!(expected_features, computed_features);
    }

    #[test]
    fn offset_works() {
        // Given
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features_offsetters: vec![
                FeatureOffsetter {
                    offsets: vec![-2, 0, 3],
                    feature: Box::new(IsDigitFeature {}) as Box<_>
                },
                FeatureOffsetter {
                    offsets: vec![-1, 1],
                    feature: Box::new(LengthFeature{}) as Box<_>
                },
            ],
        };
        let tokens = tokenize("I prefer 7 over 777", language);

        // When
        let computed_features = fp.compute_features(&tokens.as_slice());

        // Then
        let expected_features = vec![
            vec![
                ("length[+1]".to_string(), "6".to_string())
            ],
            vec![
                ("is_digit[+3]".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "1".to_string()),
                ("length[+1]".to_string(), "1".to_string())
            ],
            vec![
                ("is_digit".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "6".to_string()),
                ("length[+1]".to_string(), "4".to_string())
            ],
            vec![
                ("length[-1]".to_string(), "1".to_string()),
                ("length[+1]".to_string(), "3".to_string())
            ],
            vec![
                ("is_digit[-2]".to_string(), "1".to_string()),
                ("is_digit".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "4".to_string())
            ],
        ];
        assert_eq!(expected_features, computed_features);
    }
}
