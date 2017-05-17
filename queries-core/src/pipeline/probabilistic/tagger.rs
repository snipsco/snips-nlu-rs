use crfsuite::Tagger as CrfTagger;

use errors::*;

use pipeline::probabilistic::feature_processor::ProbabilisticFeatureProcessor;

use pipeline::FeatureProcessor;
use preprocessing::Token;

pub struct Tagger {
    tagger: CrfTagger,
    feature_processor: ProbabilisticFeatureProcessor
}

unsafe impl Send for Tagger {}
unsafe impl Sync for Tagger {}

impl Tagger {
    pub fn get_tags(&self, tokens: &[Token]) -> Result<Vec<String>> {
        let features = self.feature_processor.compute_features(&tokens);

        Ok(self.tagger.tag(&features)?)
    }

    pub fn new(crf_data: &[u8], feature_processor: ProbabilisticFeatureProcessor) -> Result<Tagger> {
        let tagger = CrfTagger::create_from_memory(crf_data)?;
        Ok(Tagger { tagger, feature_processor })
    }
}
