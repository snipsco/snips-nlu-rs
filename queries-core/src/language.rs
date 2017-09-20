use std;
use std::result;
use nlu_utils::language::Language;
use rustling_ontology::Lang;

pub struct LanguageConfig {
    pub language: Language,
}

impl std::str::FromStr for LanguageConfig {
    type Err = String;
    fn from_str(it: &str) -> result::Result<LanguageConfig, Self::Err> {
        let language = Language::from_str(it)?;
        Ok(Self { language })
    }
}

impl LanguageConfig {
    pub fn intent_classification_clusters(&self) -> Option<&str> {
        match self {
            _ => None
        }
    }

    pub fn to_rust_lang(&self) -> Lang {
        match self.language {
            Language::EN => Lang::EN,
            Language::FR => Lang::FR,
            Language::DE => Lang::DE,
            Language::KO => Lang::KO,
            Language::ES => Lang::ES,
        }
    }
}