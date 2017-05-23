use std::sync;

use crfsuite::Tagger as CrfTagger;

use errors::*;
use pipeline::FeatureProcessor;
use pipeline::probabilistic::feature_processor::ProbabilisticFeatureProcessor;
use preprocessing::Token;
use super::crf_utils::TaggingScheme;
use super::configuration::TaggerConfiguration;

pub struct Tagger {
    pub tagging_scheme: TaggingScheme,
    tagger: sync::Mutex<CrfTagger>,
    feature_processor: ProbabilisticFeatureProcessor,
}

impl Tagger {
    pub fn new(config: TaggerConfiguration) -> Result<Self> {
        unimplemented!()
    }

    pub fn new_deprecated(crf_data: &[u8], tagging_scheme: TaggingScheme, feature_processor: ProbabilisticFeatureProcessor) -> Result<Tagger> {
        let tagger = CrfTagger::create_from_memory(crf_data)?;
        Ok(Tagger { tagger: sync::Mutex::new(tagger), tagging_scheme, feature_processor })
    }

    pub fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>> {
        let features = self.feature_processor.compute_features(&tokens);
        Ok(self.tagger.lock()?.tag(&features)?)
    }
}
