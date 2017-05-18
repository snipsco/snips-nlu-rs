use crfsuite::Tagger as CrfTagger;

use errors::*;
use pipeline::FeatureProcessor;
use pipeline::probabilistic::feature_processor::ProbabilisticFeatureProcessor;
use preprocessing::Token;
use super::crf_utils::TaggingScheme;

pub struct Tagger {
    pub tagging_scheme: TaggingScheme,
    tagger: CrfTagger,
    feature_processor: ProbabilisticFeatureProcessor,
}

unsafe impl Send for Tagger {}
unsafe impl Sync for Tagger {}

impl Tagger {
    pub fn new(crf_data: &[u8], tagging_scheme: TaggingScheme, feature_processor: ProbabilisticFeatureProcessor) -> Result<Tagger> {
        let tagger = CrfTagger::create_from_memory(crf_data)?;
        Ok(Tagger { tagger, tagging_scheme, feature_processor })
    }

    pub fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>> {
        let features = self.feature_processor.compute_features(&tokens);

        Ok(self.tagger.tag(&features)?)
    }
}
