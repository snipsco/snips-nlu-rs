use errors::*;
use resources_packed::word_cluster;
use nlu_utils::language::Language;

pub trait WordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct StaticMapWordClusterer {
    language: Language,
    cluster_name: String
}

impl StaticMapWordClusterer {
    pub fn new(language: Language, cluster_name: String) -> Result<Self> {
        // Hack to check that the word cluster exists
        word_cluster(&cluster_name, language, "")?;
        Ok(
            Self {
                language: language,
                cluster_name: cluster_name
            }
        )
    }
}

impl WordClusterer for StaticMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        // Checked during initialization
        word_cluster(&self.cluster_name, self.language, word).unwrap()
    }
}
