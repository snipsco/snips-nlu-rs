use rayon::prelude::*;

use pipeline::Probability;
use pipeline::intent_classifier::{IntentClassifier, ProtobufIntentClassifier};
use models::model::Model;
use preprocessing::preprocess;

pub trait IntentParser {
    fn parse(&self, text: &str) -> Vec<Probability>;
}

pub struct ProtobufIntentParser {
    models: Vec<Model>,
}

impl ProtobufIntentParser {
    pub fn new(models: Vec<Model>) -> ProtobufIntentParser {
        ProtobufIntentParser { models: models }
    }
}

impl IntentParser for ProtobufIntentParser {
    fn parse(&self, text: &str) -> Vec<Probability> {
        let preprocessed_result = preprocess(text);

        let probabilities = self.models
            .par_iter()
            .map(|model| ProtobufIntentClassifier::new(&model).run(&preprocessed_result))
            .collect();

        probabilities
    }
}
