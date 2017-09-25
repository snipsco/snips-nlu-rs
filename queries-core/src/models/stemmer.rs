use resources_packed::stem as resource_stem;
use errors::*;

use nlu_utils::language::Language;

pub trait Stemmer: Send + Sync {
    fn stem(&self, value: &str) -> String;
}

pub struct StaticMapStemmer {
    language: Language
}

impl StaticMapStemmer {
    pub fn new(language: Language) -> Result<Self> {
        // Hack to check if stemming is supported in this language
        resource_stem(language, "")?;
        Ok(Self { language: language.clone() })
    }
}

impl Stemmer for StaticMapStemmer {
    fn stem(&self, value: &str) -> String {
        // checked during initialization
        resource_stem(self.language, value).unwrap()
    }
}
