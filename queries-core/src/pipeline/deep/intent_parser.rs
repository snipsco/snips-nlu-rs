use std::cmp::Ordering;
use std::collections::HashMap;

use rayon::prelude::*;

use errors::*;
use config::AssistantConfig;
use pipeline::{ClassifierWrapper, IntentClassifierResult, IntentParser, IntentParserResult, Slots};
use preprocessing::{DeepPreprocessor, Preprocessor, PreprocessorResult};
use super::intent_configuration::IntentConfiguration;
use super::slot_filler::compute_slots;
use yolo::Yolo;

pub struct DeepIntentParser {
    classifiers: HashMap<String, IntentConfiguration>,
    preprocessors: HashMap<String, DeepPreprocessor>,
}

impl DeepIntentParser {
    pub fn new(assistant_config: &AssistantConfig) -> Result<DeepIntentParser> {
        let mut classifiers = HashMap::new();
        let mut preprocessors = HashMap::new();

        for ref c in assistant_config.get_available_intents_names()? {
            let intent_config = assistant_config.get_intent_configuration(c)?;
            let intent = IntentConfiguration::new(intent_config)?;

            if !preprocessors.contains_key(&intent.language) {
                // Preprocessor is heavy to build, ensure we don't build it multiple time.
                preprocessors.insert(intent.language.clone(), DeepPreprocessor::new(&intent.language)?);
            }

            classifiers.insert(intent.intent_name.to_string(), intent);
        }

        Ok(DeepIntentParser { preprocessors: preprocessors, classifiers: classifiers })
    }
}

impl IntentParser for DeepIntentParser {
    fn parse(&self, input: &str, probability_threshold: f32) -> Result<Option<IntentParserResult>> {
        let preprocessor_results: Result<_> = self.preprocessors
            .iter()
            .map(|(lang, preprocessor)| Ok((&**lang, preprocessor.run(input)?)))
            .collect();
        let preprocessor_results = preprocessor_results?;

        let classif_results = get_intent(&preprocessor_results, &self.classifiers, probability_threshold)?;

        if let Some(best_classif) = classif_results.first() {
            let intent_name = best_classif.intent_name.to_string();
            let likelihood = best_classif.probability;
            let intent_configuration = self.classifiers.get(&intent_name).yolo();

            let language = &intent_configuration.language;
            let preprocessor_result = preprocessor_results.get::<str>(language).yolo();

            let slots = get_entities(&preprocessor_result, intent_configuration)?;

            Ok(Some(IntentParserResult { input: input.to_string(), likelihood, intent_name, slots }))
        } else {
            Ok(None)
        }
    }

    fn get_intent(&self,
                  input: &str,
                  probability_threshold: f32)
                  -> Result<Vec<IntentClassifierResult>> {
        ensure!(probability_threshold >= 0.0 && probability_threshold <= 1.0,
                "probability_threshold must be between 0.0 and 1.0");

        let preprocessor_results: Result<_> = self.preprocessors
            .iter()
            .map(|(lang, preprocessor)| Ok((&**lang, preprocessor.run(input)?)))
            .collect();

        get_intent(&preprocessor_results?, &self.classifiers, probability_threshold)
    }

    fn get_entities(&self, input: &str, intent_name: &str) -> Result<Slots> {
        let intent_configuration = self.classifiers
            .get(intent_name)
            .ok_or(format!("intent {:?} not found", intent_name))?;

        let language = &intent_configuration.language;
        let preprocessor = self.preprocessors.get(language).yolo();
        let preprocessor_result = preprocessor.run(input)?;

        get_entities(&preprocessor_result, &intent_configuration)
    }
}

fn get_intent(preprocessor_results: &HashMap<&str, PreprocessorResult>,
              classifiers: &HashMap<String, IntentConfiguration>,
              probability_threshold: f32)
              -> Result<Vec<IntentClassifierResult>> {
    assert!(probability_threshold >= 0.0 && probability_threshold <= 1.0,
            "probability_threshold must be between 0.0 and 1.0");

    let mut probabilities: Vec<IntentClassifierResult> = classifiers
        .par_iter()
        .map(|(_, intent_configuration)| {
            let language = &intent_configuration.language;
            let intent_name = &intent_configuration.intent_name;
            let preprocessor_result = preprocessor_results.get::<str>(language).yolo();
            let probability = intent_configuration.intent_classifier.run(&preprocessor_result);

            IntentClassifierResult {
                intent_name: intent_name.to_string(),
                probability: probability.unwrap_or_else(|e| {
                    println!("could not run intent classifier for {} : {:?}", intent_name, e);
                    -1.0
                }),
            }
        })
        .filter(|result| result.probability >= probability_threshold)
        .collect();

    probabilities.sort_by(|a, b| {
        a.probability.partial_cmp(&b.probability).unwrap_or(Ordering::Equal).reverse()
    });

    Ok(probabilities)
}

fn get_entities(preprocessor_result: &PreprocessorResult,
                intent_configuration: &IntentConfiguration)
                -> Result<Slots> {
    let predictions = intent_configuration.tokens_classifier.run(&preprocessor_result)?;

    let slot_names = &intent_configuration.slot_names;
    let slot_values = &compute_slots(&preprocessor_result, slot_names.len(), &predictions);

    let mut result = HashMap::new();
    for (name, value) in slot_names.iter().zip(slot_values.iter()) {
        result.insert(name.clone(), value.clone());
    }

    Ok(result)
}
