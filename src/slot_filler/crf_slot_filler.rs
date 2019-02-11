use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crfsuite::Tagger as CRFSuiteTagger;
use failure::{format_err, ResultExt};
use itertools::Itertools;
use snips_nlu_ontology::Language;
use snips_nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_utils::token::{tokenize, Token};

use crate::errors::*;
use crate::language::FromLanguage;
use crate::models::SlotFillerModel;
use crate::resources::SharedResources;
use crate::slot_filler::crf_utils::*;
use crate::slot_filler::feature_processor::ProbabilisticFeatureProcessor;
use crate::slot_filler::SlotFiller;
use crate::slot_utils::*;
use crate::utils::{EntityName, SlotName};

pub struct CRFSlotFiller {
    language: Language,
    tagging_scheme: TaggingScheme,
    tagger: Option<Mutex<CRFSuiteTagger>>,
    feature_processor: Option<ProbabilisticFeatureProcessor>,
    slot_name_mapping: HashMap<SlotName, EntityName>,
}

impl CRFSlotFiller {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        shared_resources: Arc<SharedResources>,
    ) -> Result<Self> {
        let slot_filler_model_path = path.as_ref().join("slot_filler.json");
        let model_file = fs::File::open(&slot_filler_model_path).with_context(|_| {
            format!(
                "Cannot open CRFSlotFiller file '{:?}'",
                &slot_filler_model_path
            )
        })?;
        let model: SlotFillerModel = serde_json::from_reader(model_file)
            .with_context(|_| "Cannot deserialize CRFSlotFiller json data")?;

        let tagging_scheme = TaggingScheme::from_u8(model.config.tagging_scheme)?;
        let slot_name_mapping = model.slot_name_mapping;
        let (tagger, feature_processor) =
            if let Some(crf_model_file) = model.crf_model_file.as_ref() {
                let crf_path = path.as_ref().join(crf_model_file);
                let tagger = CRFSuiteTagger::create_from_file(&crf_path).with_context(|_| {
                    format!("Cannot create CRFSuiteTagger from file '{:?}'", &crf_path)
                })?;
                let feature_processor = ProbabilisticFeatureProcessor::new(
                    &model.config.feature_factory_configs,
                    shared_resources.clone(),
                )?;
                (Some(Mutex::new(tagger)), Some(feature_processor))
            } else {
                (None, None)
            };
        let language = Language::from_str(&model.language_code)?;

        Ok(Self {
            language,
            tagging_scheme,
            tagger,
            feature_processor,
            slot_name_mapping,
        })
    }
}

impl SlotFiller for CRFSlotFiller {
    fn get_tagging_scheme(&self) -> TaggingScheme {
        self.tagging_scheme
    }

    fn get_slots(&self, text: &str) -> Result<Vec<InternalSlot>> {
        if let (Some(ref tagger), Some(ref feature_processor)) =
            (self.tagger.as_ref(), self.feature_processor.as_ref())
        {
            let tokens = tokenize(text, NluUtilsLanguage::from_language(self.language));
            if tokens.is_empty() {
                return Ok(vec![]);
            }
            let features = feature_processor.compute_features(&&*tokens)?;
            let tags = tagger
                .lock()
                .map_err(|e| format_err!("Poisonous mutex: {}", e))?
                .tag(&features)?
                .into_iter()
                .map(|tag| decode_tag(&*tag))
                .collect::<Result<Vec<String>>>()?;

            tags_to_slots(
                text,
                &tokens,
                &tags,
                self.tagging_scheme,
                &self.slot_name_mapping,
            )
        } else {
            Ok(vec![])
        }
    }

    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64> {
        if let (Some(ref tagger), Some(ref feature_processor)) =
            (self.tagger.as_ref(), self.feature_processor.as_ref())
        {
            let features = feature_processor.compute_features(&tokens)?;
            let tagger = tagger
                .lock()
                .map_err(|e| format_err!("poisonous mutex: {}", e))?;
            let tagger_labels = tagger
                .labels()?
                .into_iter()
                .map(|label| decode_tag(&*label))
                .collect::<Result<Vec<String>>>()?;
            let tagger_labels_slice = tagger_labels.iter().map(|l| &**l).collect_vec();
            // Substitute tags that were not seen during training
            let cleaned_tags = tags
                .iter()
                .map(|t| {
                    if tagger_labels.contains(t) {
                        t
                    } else {
                        get_substitution_label(&*tagger_labels_slice)
                    }
                })
                .map(|t| encode_tag(t))
                .collect_vec();
            tagger.set(&features)?;
            Ok(tagger.probability(&cleaned_tags)?)
        } else {
            // No tagger defined corresponds to an intent without slots
            Ok(tags
                .into_iter()
                .find(|tag| tag != OUTSIDE)
                .map(|_| 0.0)
                .unwrap_or(1.0))
        }
    }
}

impl CRFSlotFiller {
    pub fn compute_features(&self, text: &str) -> Result<Vec<Vec<(String, String)>>> {
        let tokens = tokenize(text, NluUtilsLanguage::from_language(self.language));
        if tokens.is_empty() {
            return Ok(vec![]);
        };
        Ok(
            if let Some(feature_processor) = self.feature_processor.as_ref() {
                feature_processor.compute_features(&&*tokens)?
            } else {
                tokens.iter().map(|_| vec![]).collect()
            },
        )
    }
}

// We need to use base64 encoding to ensure ascii encoding because of encoding issues in
// python-crfsuite

fn decode_tag(tag: &str) -> Result<String> {
    let bytes = base64::decode(tag)?;
    Ok(String::from_utf8(bytes)?)
}

fn encode_tag(tag: &str) -> String {
    base64::encode(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::resources::loading::load_engine_shared_resources;

    #[test]
    fn test_load_from_path() {
        // Given
        let trained_engine_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine");

        let slot_filler_path = trained_engine_path
            .join("probabilistic_intent_parser")
            .join("slot_filler_0");

        let resources = load_engine_shared_resources(trained_engine_path).unwrap();

        // When
        let slot_filler = CRFSlotFiller::from_path(slot_filler_path, resources).unwrap();
        let slots = slot_filler.get_slots("make me two cups of coffee").unwrap();

        // Then
        let expected_slots = vec![InternalSlot {
            value: "two".to_string(),
            char_range: 8..11,
            entity: "snips/number".to_string(),
            slot_name: "number_of_cups".to_string(),
        }];
        assert_eq!(expected_slots, slots);
    }
}
