use ndarray::prelude::*;

use errors::*;

use pipeline::{Probability, Prediction};
use super::tf_classifier_wrapper::TFClassifierWrapper;
use config::ArcBoxedIntentConfig;

pub struct IntentConfiguration {
    pub intent_classifier: TFClassifierWrapper<Probability>,
    pub tokens_classifier: TFClassifierWrapper<Array1<Prediction>>,
    pub slot_names: Vec<String>,
    pub intent_name: String,
}

impl IntentConfiguration {
    pub fn new(intent_config: ArcBoxedIntentConfig) -> Result<IntentConfiguration> {
        let data = intent_config.get_pb_config()?;
        let slots = data.get_slots().iter().map(|s| s.get_name().to_string()).collect();

        Ok(IntentConfiguration {
            intent_classifier: TFClassifierWrapper::new_intent_classifier(intent_config.clone())?,
            tokens_classifier: TFClassifierWrapper::new_tokens_classifier(intent_config.clone())?,
            intent_name: data.name.clone(),
            slot_names: slots,
        })
    }
}
