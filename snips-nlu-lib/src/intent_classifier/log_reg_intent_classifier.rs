use std::collections::HashSet;

use itertools::Itertools;
use ndarray::prelude::*;

use models::IntentClassifierModel;
use errors::*;
use intent_classifier::logreg::MulticlassLogisticRegression;
use intent_classifier::{Featurizer, IntentClassifier};
use snips_nlu_ontology::IntentClassifierResult;

pub struct LogRegIntentClassifier {
    intent_list: Vec<Option<String>>,
    featurizer: Option<Featurizer>,
    logreg: Option<MulticlassLogisticRegression>,
}

impl LogRegIntentClassifier {
    pub fn new(config: IntentClassifierModel) -> Result<Self> {
        let featurizer: Option<Featurizer> = if let Some(featurizer_config) = config.featurizer {
            Some(Featurizer::new(featurizer_config)?)
        } else {
            None
        };

        let logreg = if let (Some(intercept), Some(coeffs)) = (config.intercept, config.coeffs) {
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
            intent_list: config.intent_list,
            featurizer,
            logreg,
        })
    }
}

impl IntentClassifier for LogRegIntentClassifier {
    fn get_intent(
        &self,
        input: &str,
        intents_filter: Option<&HashSet<String>>,
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
            let filtered_out_indexes = get_filtered_out_intents_indexes(&self.intent_list, intents_filter);
            let probabilities = logreg.run(&features.view(), filtered_out_indexes)?;

            let mut intents_proba: Vec<(&Option<String>, &f32)> = self.intent_list
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
    intents_list: &Vec<Option<String>>,
    intents_filter: Option<&HashSet<String>>,
) -> Option<Vec<usize>> {
    intents_filter.map(|filter|
        intents_list
            .into_iter()
            .enumerate()
            .filter_map(|(i, opt_intent)|
                if let Some(intent) = opt_intent.as_ref() {
                    if !filter.contains(intent) {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            )
            .collect()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use models::{FeaturizerConfiguration, FeaturizerModel, TfIdfVectorizerModel};

    fn get_sample_log_reg_classifier() -> LogRegIntentClassifier {
        let language_code = "en".to_string();
        let best_features = vec![
            1, 2, 15, 17, 19, 20, 21, 22, 28, 30, 36, 37, 44, 45, 47, 54, 55, 68, 72, 73, 82, 92,
            93, 96, 97, 100, 101,
        ];
        let entity_utterances_to_feature_names = hashmap![];
        let vocab = hashmap![
            "!".to_string() => 0,
            "12".to_string() => 1,
            "?".to_string() => 2,
            "a".to_string() => 3,
            "about".to_string() => 4,
            "agent".to_string() => 5,
            "albuquerque".to_string() => 6,
            "and".to_string() => 7,
            "ask".to_string() => 8,
            "assume".to_string() => 9,
            "at".to_string() => 10,
            "be".to_string() => 11,
            "believe".to_string() => 12,
            "border".to_string() => 13,
            "break".to_string() => 14,
            "brew".to_string() => 15,
            "buena".to_string() => 16,
            "can".to_string() => 17,
            "center".to_string() => 18,
            "coffe".to_string() => 19,
            "coffees".to_string() => 20,
            "cold".to_string() => 21,
            "cup".to_string() => 22,
            "do".to_string() => 23,
            "down".to_string() => 24,
            "easi".to_string() => 25,
            "feel".to_string() => 26,
            "fellas".to_string() => 27,
            "five".to_string() => 28,
            "for".to_string() => 29,
            "four".to_string() => 30,
            "france".to_string() => 31,
            "fun".to_string() => 32,
            "game".to_string() => 33,
            "gather".to_string() => 34,
            "georgina".to_string() => 35,
            "get".to_string() => 36,
            "give".to_string() => 37,
            "going".to_string() => 38,
            "he".to_string() => 39,
            "hear".to_string() => 40,
            "here".to_string() => 41,
            "him".to_string() => 42,
            "hollywood".to_string() => 43,
            "hot".to_string() => 44,
            "hundr".to_string() => 45,
            "i".to_string() => 46,
            "iced".to_string() => 47,
            "in".to_string() => 48,
            "it".to_string() => 49,
            "kind".to_string() => 50,
            "lassy".to_string() => 51,
            "like".to_string() => 52,
            "m".to_string() => 53,
            "make".to_string() => 54,
            "me".to_string() => 55,
            "miller".to_string() => 56,
            "miltan".to_string() => 57,
            "my".to_string() => 58,
            "n".to_string() => 59,
            "newhouse".to_string() => 60,
            "no".to_string() => 61,
            "of".to_string() => 62,
            "off".to_string() => 63,
            "offended".to_string() => 64,
            "offic".to_string() => 65,
            "okay".to_string() => 66,
            "on".to_string() => 67,
            "one".to_string() => 68,
            "orlando".to_string() => 69,
            "patrol".to_string() => 70,
            "plane".to_string() => 71,
            "please".to_string() => 72,
            "prepare".to_string() => 73,
            "prostitutes".to_string() => 74,
            "realli".to_string() => 75,
            "ribs".to_string() => 76,
            "roger".to_string() => 77,
            "s".to_string() => 78,
            "scrapple".to_string() => 79,
            "scumbag".to_string() => 80,
            "she".to_string() => 81,
            "six".to_string() => 82,
            "someth".to_string() => 83,
            "sound".to_string() => 84,
            "special".to_string() => 85,
            "states".to_string() => 86,
            "strike".to_string() => 87,
            "studio".to_string() => 88,
            "suerte".to_string() => 89,
            "t".to_string() => 90,
            "take".to_string() => 91,
            "tea".to_string() => 92,
            "teas".to_string() => 93,
            "the".to_string() => 94,
            "think".to_string() => 95,
            "thousand".to_string() => 96,
            "three".to_string() => 97,
            "to".to_string() => 98,
            "truth".to_string() => 99,
            "twenti".to_string() => 100,
            "two".to_string() => 101,
            "united".to_string() => 102,
            "well".to_string() => 103,
            "what".to_string() => 104,
            "when".to_string() => 105,
            "whew".to_string() => 106,
            "why".to_string() => 107,
            "with".to_string() => 108,
            "wo".to_string() => 109,
            "would".to_string() => 110,
            "wow".to_string() => 111,
            "you".to_string() => 112,
        ];

        let idf_diag = vec![
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.27726728501,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.27726728501,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            2.71765149707,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.0541237337,
            3.97041446557,
            3.97041446557,
            2.58412010445,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.27726728501,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.27726728501,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            2.71765149707,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            2.71765149707,
            2.46633706879,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            2.8718021769,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.56494935746,
            3.27726728501,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.0541237337,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.56494935746,
            3.56494935746,
            3.27726728501,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.27726728501,
            3.97041446557,
            3.56494935746,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.56494935746,
            3.97041446557,
            3.97041446557,
            3.97041446557,
            3.27726728501,
        ];

        let tfidf_vectorizer = TfIdfVectorizerModel { idf_diag, vocab };

        let intent_list: Vec<Option<String>> = vec![
            Some("MakeCoffee".to_string()),
            Some("MakeTea".to_string()),
            None,
        ];

        let config = FeaturizerConfiguration {
            sublinear_tf: false,
            word_clusters_name: None,
        };

        let config = FeaturizerModel {
            tfidf_vectorizer,
            best_features,
            config,
            language_code,
            entity_utterances_to_feature_names,
        };

        let featurizer = Featurizer::new(config).unwrap();

        let intercept = array![
            -0.6769558144299883,
            -0.6587242944035958,
            0.22680835693804338
        ];

        let coeffs_vec = vec![
            [
                0.47317020196399323,
                -0.38075250099680313,
                1.107799468598624,
                -0.38075250099680313,
                1.8336263975786775,
                0.8353246023070073,
                -0.38075250099680313,
                2.2249713330204766,
                0.08564143623516322,
                0.5332023901777503,
                -0.38075250099680313,
                0.8353246023070073,
                -0.550417616014284,
                0.7005943889737921,
                -0.6161745296811834,
                0.7232703408462136,
                1.5548021356237207,
                0.26001735853448454,
                0.40815046754904194,
                -0.550417616014284,
                0.8353246023070073,
                -1.4480803940924434,
                -0.8951192396337332,
                0.47613450034233684,
                0.30011894863821786,
                0.24107723670655656,
                0.07579876754730583,
            ],
            [
                -0.36011489995898516,
                0.9544411862213601,
                -0.6209197902493954,
                0.9544411862213601,
                -1.3347876038937607,
                -0.45132716150922075,
                0.9544411862213601,
                -1.144908928720865,
                0.4753730257377091,
                -0.25761552096599194,
                0.9544411862213601,
                -0.45132716150922075,
                1.2004101968975385,
                -0.43392555576901004,
                1.2094993585173603,
                0.6986318740136787,
                -1.0131190277108526,
                0.7937664891170565,
                0.45173521169661446,
                1.2004101968975385,
                -0.45132716150922075,
                2.9446608222158592,
                1.9429554575341705,
                -0.42500086360353684,
                0.3681826115884594,
                0.3763435734118238,
                0.696370959190279,
            ],
            [
                -0.3208821394723137,
                -0.4047461312958966,
                -0.73500565414034,
                -0.4047461312958966,
                -0.9726774017143353,
                -0.46703967551075193,
                -0.4047461312958966,
                -1.5028381667964201,
                -0.5558940158035158,
                -0.4547178634891068,
                -0.4047461312958966,
                -0.46703967551075193,
                -0.424271594788462,
                -0.3638848118522113,
                -0.4134263927057856,
                -1.3356856351554096,
                -1.2356655443188445,
                -0.7929704501312185,
                -0.782757638722614,
                -0.424271594788462,
                -0.46703967551075193,
                -0.8045518775902378,
                -0.7346194305470242,
                -0.21437336251972489,
                -0.61116631674614,
                -0.6014286441350187,
                -0.6979309347340573,
            ],
        ];

        let coeffs: Array2<f32> = Array::from_shape_fn((27, 3), |(i, j)| coeffs_vec[j][i]);
        let logreg = MulticlassLogisticRegression::new(intercept, coeffs).unwrap();
        LogRegIntentClassifier {
            featurizer: Some(featurizer),
            intent_list,
            logreg: Some(logreg),
        }
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
            probability: 0.6514961,
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
            None
        ];
        let intents_filter = hashset!["intent1".to_string(), "intent3".to_string()];

        // When
        let filtered_indexes = get_filtered_out_intents_indexes(&intents_list, Some(&intents_filter));

        // Then
        let expected_indexes = Some(vec![1]);
        assert_eq!(expected_indexes, filtered_indexes);
    }
}
