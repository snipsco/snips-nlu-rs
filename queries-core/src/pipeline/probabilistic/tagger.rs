use std::sync;

use crfsuite::Tagger as CRFTagger;

use errors::*;
use pipeline::FeatureProcessor;
use pipeline::probabilistic::feature_processor::ProbabilisticFeatureProcessor;
use preprocessing::Token;
use base64::decode;
use super::crf_utils::TaggingScheme;
use super::configuration::TaggerConfiguration;

pub struct Tagger {
    pub tagging_scheme: TaggingScheme,
    tagger: sync::Mutex<CRFTagger>,
    feature_processor: ProbabilisticFeatureProcessor,
}

impl Tagger {
    pub fn new(config: TaggerConfiguration) -> Result<Tagger> {
        let tagging_scheme = TaggingScheme::from_u8(config.tagging_scheme)?;
        let feature_processor = ProbabilisticFeatureProcessor::new(&config.features_signatures)?;
        let converted_data = decode(&config.crf_model_data)?;
        let tagger = CRFTagger::create_from_memory(converted_data)?;
        Ok(Self { tagging_scheme, tagger: sync::Mutex::new(tagger), feature_processor })
    }

    pub fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>> {
        let features = self.feature_processor.compute_features(&tokens);
        Ok(self.tagger.lock()?.tag(&features)?)
    }

    pub fn get_sequence_probability(&self, tokens: &[Token], tags: Vec<String>) -> Result<f64> {
        let features = self.feature_processor.compute_features(&tokens);
        let tagger = self.tagger.lock()?;
        tagger.set(&features)?;
        Ok(tagger.probability(tags)?)
    }
}
