use std::f32;

use ndarray::prelude::*;

use errors::*;
use models::logreg::MulticlassLogisticRegression;
use pipeline::IntentClassifierResult;
use super::feature_processor::FeatureProcessor;

pub struct IntentClassifier {
    intent_list: Vec<Option<String>>,
    intercept: Array1<f32>,
    coeff: Array2<f32>,
    featurizer: FeatureProcessor,
    logreg: MulticlassLogisticRegression,
}

impl IntentClassifier {
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn get_intent(&self, input: &str) -> Result<Vec<IntentClassifierResult>> {
        if input.is_empty() || self.intent_list.is_empty() {
            return Ok(vec![]);
        }

        if self.intent_list.len() == 1 {
            return if let Some(ref intent_name) = self.intent_list[0] {
                Ok(vec![IntentClassifierResult { intent_name: intent_name.clone(), probability: 1.0 }])
            } else {
                Ok(vec![])
            }
        }

        let stemmed_text = input;

        let x = self.featurizer.transform(stemmed_text);
        let probabilities = self.logreg.run(&x.view())?;

        let (index_predicted, best_probability) = argmax(&probabilities);

        if let Some(ref intent_name) = self.intent_list[index_predicted] {
            Ok(vec![IntentClassifierResult { intent_name: intent_name.clone(), probability: best_probability }])
        } else {
            Ok(vec![])
        }
    }
}

fn argmax(arr: &Array1<f32>) -> (usize, f32) {
    let mut index = 0;
    let mut max_value = f32::NEG_INFINITY;
    for (j, &value) in arr.iter().enumerate() {
        if value > max_value {
            index = j;
            max_value = value;
        }
    }
    (index, max_value)
}
