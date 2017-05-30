use errors::*;
use resources_packed::word_cluster;

pub trait WordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct StaticMapWordClusterer {
    language_code: String,
    cluster_name: String
}

impl StaticMapWordClusterer {
    pub fn new(language_code: String, cluster_name: String) -> Result<Self> {
        // Hack to check that the word cluster exists
        word_cluster(&cluster_name, &language_code, "")?;
        Ok(
            Self {
                language_code: language_code,
                cluster_name: cluster_name
            }
        )
    }
}

impl WordClusterer for StaticMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        // Checked during initialization
        word_cluster(&self.cluster_name, &self.language_code, word).unwrap()
    }
}
