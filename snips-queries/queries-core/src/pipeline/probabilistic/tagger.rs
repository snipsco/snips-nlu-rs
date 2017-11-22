use std::sync;

use crfsuite::Tagger as CRFSuiteTagger;
use itertools::Itertools;

use errors::*;
use pipeline::FeatureProcessor;
use pipeline::probabilistic::feature_processor::ProbabilisticFeatureProcessor;
use nlu_utils::token::Token;
use base64::decode;
use super::crf_utils::{get_substitution_label, TaggingScheme};
use super::configuration::TaggerConfiguration;

pub trait Tagger: Send + Sync {
    fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>>;
    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64>;
    fn get_tagging_scheme(&self) -> TaggingScheme;
}

pub struct CRFTagger {
    tagging_scheme: TaggingScheme,
    tagger: sync::Mutex<CRFSuiteTagger>,
    feature_processor: ProbabilisticFeatureProcessor,
}

impl Tagger for CRFTagger {
    fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>> {
        let features = self.feature_processor.compute_features(&tokens);
        Ok(self.tagger.lock()?.tag(&features)?)
    }

    fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64> {
        let features = self.feature_processor.compute_features(&tokens);
        let tagger = self.tagger.lock()?;
        let tagger_labels = tagger.labels()?;
        let tagger_labels_slice = tagger_labels.iter().map(|l| &**l).collect_vec();
        // Substitute tags that were not seen during training
        let cleaned_tags = tags.into_iter()
            .map(|t|
                if tagger_labels.contains(&t) {
                    t
                } else {
                    get_substitution_label(&*tagger_labels_slice)
                })
            .collect_vec();
        tagger.set(&features)?;
        Ok(tagger.probability(cleaned_tags)?)
    }
    fn get_tagging_scheme(&self) -> TaggingScheme {
        self.tagging_scheme
    }
}

impl CRFTagger {
    pub fn new(config: TaggerConfiguration) -> Result<CRFTagger> {
        let tagging_scheme = TaggingScheme::from_u8(config.tagging_scheme)?;
        let feature_processor = ProbabilisticFeatureProcessor::new(&config.features_signatures)?;
        let converted_data = decode(&config.crf_model_data)?;
        let tagger = CRFSuiteTagger::create_from_memory(converted_data)?;
        Ok(Self { tagging_scheme, tagger: sync::Mutex::new(tagger), feature_processor })
    }
}
