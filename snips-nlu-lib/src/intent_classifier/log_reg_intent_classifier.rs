use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use failure::ResultExt;
use itertools::Itertools;
use ndarray::prelude::*;
use snips_nlu_ontology::IntentClassifierResult;

use crate::errors::*;
use crate::intent_classifier::{Featurizer, IntentClassifier};
use crate::models::IntentClassifierModel;
use crate::resources::SharedResources;
use crate::utils::IntentName;

use super::logreg::MulticlassLogisticRegression;

pub struct LogRegIntentClassifier {
    intent_list: Vec<Option<IntentName>>,
    featurizer: Option<Featurizer>,
    logreg: Option<MulticlassLogisticRegression>,
}

impl LogRegIntentClassifier {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let classifier_model_path = path.as_ref().join("intent_classifier.json");
        let model_file = File::open(&classifier_model_path).with_context(|_| {
            format!(
                "Cannot open LogRegIntentClassifier file '{:?}'",
                &classifier_model_path
            )
        })?;
        let model: IntentClassifierModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize LogRegIntentClassifier json data")?;

        let featurizer: Option<Featurizer> = if let Some(featurizer_name) = model.featurizer {
            let featurizer_path = path.as_ref().join(&featurizer_name);
            Some(Featurizer::from_path(&featurizer_path, shared_resources)?)
        } else {
            None
        };

        let logreg = if let (Some(intercept), Some(coeffs)) = (model.intercept, model.coeffs) {
            let arr_intercept = Array::from_vec(intercept);
            let nb_classes = arr_intercept.dim();
            let nb_features = coeffs[0].len();
            // Note: the deserialized coeffs matrix is transposed
            let arr_weights =
                Array::from_shape_fn((nb_features, nb_classes), |(i, j)| coeffs[j][i]);
            MulticlassLogisticRegression::new(arr_intercept, arr_weights).map(Some)
        } else {
            Ok(None)
        }?;

        Ok(Self {
            intent_list: model.intent_list,
            featurizer,
            logreg,
        })
    }
}

impl IntentClassifier for LogRegIntentClassifier {
    fn get_intent(
        &self,
        input: &str,
        intents_filter: Option<&HashSet<IntentName>>,
    ) -> Result<Option<IntentClassifierResult>> {
        if input.is_empty() || self.intent_list.is_empty() {
            return Ok(None);
        }

        if self.intent_list.len() == 1 {
            return Ok(self.intent_list[0]
                .as_ref()
                .map(|intent_name| IntentClassifierResult {
                    intent_name: intent_name.clone(),
                    probability: 1.0,
                }));
        }

        if let (Some(featurizer), Some(logreg)) = (self.featurizer.as_ref(), self.logreg.as_ref()) {
            let features = featurizer.transform(input)?;
            let filtered_out_indexes =
                get_filtered_out_intents_indexes(&self.intent_list, intents_filter);
            let probabilities = logreg.run(&features.view(), filtered_out_indexes)?;

            let mut intents_proba: Vec<(&Option<IntentName>, &f32)> = self
                .intent_list
                .iter()
                .zip(probabilities.into_iter())
                .collect_vec();

            // Sort intents by decreasing probabilities
            intents_proba.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            let mut filtered_intents = intents_proba.into_iter().filter(|&(opt_intent, _)| {
                if let Some(intent) = opt_intent.as_ref() {
                    intents_filter
                        .map(|intents| intents.contains(intent))
                        .unwrap_or(true)
                } else {
                    true
                }
            });

            filtered_intents
                .next()
                .map(|(opt_intent, proba)| {
                    Ok(opt_intent
                        .clone()
                        .map(|intent_name| IntentClassifierResult {
                            intent_name: intent_name.clone(),
                            probability: *proba,
                        }))
                })
                .unwrap_or(Ok(None))
        } else {
            Ok(None)
        }
    }
}

impl LogRegIntentClassifier {
    pub fn compute_features(&self, input: &str) -> Result<Array1<f32>> {
        self.featurizer
            .as_ref()
            .map(|featurizer| featurizer.transform(input))
            .unwrap_or_else(|| Ok(Array::from_iter(vec![])))
    }
}

fn get_filtered_out_intents_indexes(
    intents_list: &[Option<IntentName>],
    intents_filter: Option<&HashSet<IntentName>>,
) -> Option<Vec<usize>> {
    intents_filter.map(|filter| {
        intents_list
            .iter()
            .enumerate()
            .filter_map(|(i, opt_intent)| {
                if let Some(intent) = opt_intent.as_ref() {
                    if !filter.contains(intent) {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::intent_classifier::TfidfVectorizer;
    use crate::models::{
        SklearnVectorizerModel, TfidfVectorizerConfiguration, TfidfVectorizerModel,
    };
    use crate::resources::loading::load_engine_shared_resources;
    use crate::testutils::*;

    fn get_sample_log_reg_classifier() -> LogRegIntentClassifier {
        let trained_engine_dir = file_path("tests").join("models").join("nlu_engine");

        let resources = load_engine_shared_resources(trained_engine_dir).unwrap();
        let language_code = "en".to_string();

        let vocab = hashmap![
            "?".to_string() => 0,
            "and".to_string() => 1,
            "boiling".to_string() => 2,
            "brew".to_string() => 3,
            "builtinentityfeaturesnipsnumber".to_string() => 4,
            "can".to_string() => 5,
            "coffee".to_string() => 6,
            "cold".to_string() => 7,
            "cup".to_string() => 8,
            "cups".to_string() => 9,
            "entityfeaturetemperature".to_string() => 10,
            "hot".to_string() => 11,
            "make".to_string() => 12,
            "me".to_string() => 13,
            "please".to_string() => 14,
            "pls".to_string() => 15,
            "prepare".to_string() => 16,
            "tea".to_string() => 17,
            "unknownword".to_string() => 18,
            "want".to_string() => 19,
            "you".to_string() => 20,
        ];

        let idf_diag = vec![
            4.15700042, 3.38381053, 4.33932198, 4.00284974, 2.36524095, 3.38381053, 2.90423745,
            4.15700042, 2.72988407, 3.11554655, 3.17617117, 3.55086462, 3.00432091, 3.00432091,
            3.75153531, 4.15700042, 3.38381053, 2.90423745, 1.15296934, 4.15700042, 3.17617117,
        ];

        let vectorizer_ = SklearnVectorizerModel { idf_diag, vocab };

        let tfidf_vectorizer_config = TfidfVectorizerConfiguration {
            use_stemming: false,
            word_clusters_name: None,
        };

        let tfidf_vectorizer_model = TfidfVectorizerModel {
            language_code,
            builtin_entity_scope: vec!["snips/number".to_string()],
            vectorizer: vectorizer_,
            config: tfidf_vectorizer_config,
        };

        let tfidf_vectorizer = TfidfVectorizer::new(tfidf_vectorizer_model, resources).unwrap();

        let intent_list: Vec<Option<String>> = vec![
            Some("MakeCoffee".to_string()),
            Some("MakeTea".to_string()),
            None,
        ];

        let featurizer = Featurizer::new(tfidf_vectorizer, None);

        let intercept = array![-0.06864156, -0.08753256, -0.05181312];

        let coeffs_vec = vec![
            [
                -0.55510086,
                -0.86491577,
                -0.27719474,
                1.01186938,
                0.90334115,
                0.45271861,
                2.40544488,
                -0.36875983,
                0.99532187,
                0.02645324,
                0.,
                -0.74874958,
                0.14024503,
                0.1279823,
                0.97976051,
                -0.36875983,
                0.45271861,
                -1.3218943,
                -2.84005242,
                -0.36875983,
                0.15139972,
            ],
            [
                0.7976861,
                -0.95653898,
                0.68651478,
                -0.70297145,
                0.05727077,
                0.02173045,
                -1.65874611,
                0.7715152,
                0.15168923,
                0.87229664,
                0.,
                1.60177833,
                0.80573202,
                0.83453344,
                -0.85976499,
                0.7715152,
                0.02173045,
                2.63325741,
                -2.38149177,
                0.7715152,
                -0.27275113,
            ],
            [
                -0.26737936,
                0.68592655,
                -0.33716278,
                -0.48817112,
                -1.27965565,
                -0.59612932,
                -0.99629522,
                -0.40549986,
                -1.12883455,
                -0.99187234,
                0.,
                -0.68324725,
                -0.9105869,
                -0.92031765,
                -0.31696237,
                -0.40549986,
                -0.59612932,
                -1.12923572,
                1.99231387,
                -0.40549986,
                -0.32869601,
            ],
        ];

        let coeffs: Array2<f32> = Array::from_shape_fn((21, 3), |(i, j)| coeffs_vec[j][i]);
        let logreg = MulticlassLogisticRegression::new(intercept, coeffs).unwrap();
        LogRegIntentClassifier {
            featurizer: Some(featurizer),
            intent_list,
            logreg: Some(logreg),
        }
    }

    #[test]
    fn from_path_works() {
        // Given
        let trained_engine_dir = file_path("tests").join("models").join("nlu_engine");

        let classifier_path = trained_engine_dir
            .join("probabilistic_intent_parser")
            .join("intent_classifier");

        let resources = load_engine_shared_resources(trained_engine_dir).unwrap();

        // When
        let intent_classifier =
            LogRegIntentClassifier::from_path(classifier_path, resources).unwrap();
        let intent_result = intent_classifier
            .get_intent("Make me one cup of tea please", None)
            .unwrap()
            .map(|res| res.intent_name);

        // Then
        let expected_intent = Some("MakeTea".to_string());
        assert_eq!(expected_intent, intent_result);
    }

    #[test]
    fn get_intent_works() {
        // Given
        let classifier = get_sample_log_reg_classifier();

        // When
        let classification_result = classifier.get_intent("Make me two cups of tea", None);
        let ref actual_result = classification_result.unwrap().unwrap();
        let expected_result = IntentClassifierResult {
            intent_name: "MakeTea".to_string(),
            probability: 0.9088109819597295,
        };

        // Then
        assert_eq!(expected_result.intent_name, actual_result.intent_name);
        assert_eq!(expected_result.probability, actual_result.probability);
    }

    #[test]
    fn should_filter_intents() {
        // Given
        let classifier = get_sample_log_reg_classifier();

        // When
        let text1 = "Make me two cups of tea";
        let result1 = classifier
            .get_intent(
                text1,
                Some(hashset! {"MakeCoffee".to_string(), "MakeTea".to_string()}).as_ref(),
            )
            .unwrap();

        let text2 = "Make me two cups of tea";
        let result2 = classifier
            .get_intent(text2, Some(hashset! {"MakeCoffee".to_string()}).as_ref())
            .unwrap();

        let text3 = "bla bla bla";
        let result3 = classifier
            .get_intent(text3, Some(hashset! {"MakeCoffee".to_string()}).as_ref())
            .unwrap();

        // Then
        assert_eq!(
            Some("MakeTea".to_string()),
            result1.map(|res| res.intent_name)
        );
        assert_eq!(
            Some("MakeCoffee".to_string()),
            result2.map(|res| res.intent_name)
        );
        assert_eq!(None, result3);
    }

    #[test]
    fn should_get_filtered_out_intents_indexes() {
        // Given
        let intents_list = vec![
            Some("intent1".to_string()),
            Some("intent2".to_string()),
            Some("intent3".to_string()),
            None,
        ];
        let intents_filter = hashset!["intent1".to_string(), "intent3".to_string()];

        // When
        let filtered_indexes =
            get_filtered_out_intents_indexes(&intents_list, Some(&intents_filter));

        // Then
        let expected_indexes = Some(vec![1]);
        assert_eq!(expected_indexes, filtered_indexes);
    }
}
