use std::collections::HashMap;
use std::sync::Arc;

use failure::bail;
use itertools::Itertools;
use snips_nlu_utils::token::Token;

use crate::errors::*;
use crate::models::FeatureFactory;
use crate::resources::SharedResources;
use crate::slot_filler::features::*;

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

        Ok(ProbabilisticFeatureProcessor {
            features_offsetters,
        })
    }
}

impl ProbabilisticFeatureProcessor {
    #[rustfmt::skip]
    pub fn compute_features(&self, input: &&[Token]) -> Result<Vec<Vec<(String, String)>>> {
        let mut features = vec![vec![]; input.len()];
        for offsetter in self.features_offsetters.iter() {
            for i in 0..input.len() {
                if let Some(value) = offsetter.feature.compute(input, i)? {
                    offsetter.offsets_with_name().iter().foreach(|&(offset, ref key)| {
                        if i as i32 - offset >= 0 && i as i32 - offset < input.len() as i32 {
                            features[(i as i32 - offset) as usize].push(
                                (key.clone(), value.clone())
                            );
                        }
                    });
                }
            }
        }
        Ok(features)
    }
}

struct FeatureOffsetter {
    feature: Box<Feature>,
    offsets: Vec<i32>,
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
    fn name(&self) -> String {
        self.feature_kind().identifier().to_string()
    }
    fn build_features(
        args: &HashMap<String, serde_json::Value>,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Vec<Box<Feature>>>
    where
        Self: Sized;
    fn compute(&self, tokens: &[Token], token_index: usize) -> Result<Option<String>>;
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
    (CustomEntityMatchFeature, entity_match),
    (BuiltinEntityMatchFeature, builtin_entity_match),
    (WordClusterFeature, word_cluster)
]);

#[cfg(test)]
mod tests {
    use super::*;

    use snips_nlu_utils::language::Language;
    use snips_nlu_utils::token::tokenize;

    #[test]
    fn test_compute_features() {
        // Given
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features_offsetters: vec![
                FeatureOffsetter {
                    offsets: vec![0],
                    feature: Box::new(IsDigitFeature {}) as Box<_>,
                },
                FeatureOffsetter {
                    offsets: vec![0],
                    feature: Box::new(LengthFeature {}) as Box<_>,
                },
            ],
        };
        let tokens = tokenize("I prefer 7 over 777", language);

        // When
        let computed_features = fp.compute_features(&tokens.as_slice()).unwrap();

        let expected_features = vec![
            vec![("length".to_string(), "1".to_string())],
            vec![("length".to_string(), "6".to_string())],
            vec![
                ("is_digit".to_string(), "1".to_string()),
                ("length".to_string(), "1".to_string()),
            ],
            vec![("length".to_string(), "4".to_string())],
            vec![
                ("is_digit".to_string(), "1".to_string()),
                ("length".to_string(), "3".to_string()),
            ],
        ];

        // Then
        assert_eq!(expected_features, computed_features);
    }

    #[test]
    fn test_offset() {
        // Given
        let language = Language::EN;
        let fp = ProbabilisticFeatureProcessor {
            features_offsetters: vec![
                FeatureOffsetter {
                    offsets: vec![-2, 0, 3],
                    feature: Box::new(IsDigitFeature {}) as Box<_>,
                },
                FeatureOffsetter {
                    offsets: vec![-1, 1],
                    feature: Box::new(LengthFeature {}) as Box<_>,
                },
            ],
        };
        let tokens = tokenize("I prefer 7 over 777", language);

        // When
        let computed_features = fp.compute_features(&tokens.as_slice()).unwrap();

        // Then
        let expected_features = vec![
            vec![("length[+1]".to_string(), "6".to_string())],
            vec![
                ("is_digit[+3]".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "1".to_string()),
                ("length[+1]".to_string(), "1".to_string()),
            ],
            vec![
                ("is_digit".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "6".to_string()),
                ("length[+1]".to_string(), "4".to_string()),
            ],
            vec![
                ("length[-1]".to_string(), "1".to_string()),
                ("length[+1]".to_string(), "3".to_string()),
            ],
            vec![
                ("is_digit[-2]".to_string(), "1".to_string()),
                ("is_digit".to_string(), "1".to_string()),
                ("length[-1]".to_string(), "4".to_string()),
            ],
        ];
        assert_eq!(expected_features, computed_features);
    }
}
