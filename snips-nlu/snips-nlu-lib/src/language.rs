use std::result::Result;
use std::str::FromStr;

use nlu_utils::language::Language;
use rustling_ontology::Lang as RustLang;

pub struct LanguageConfig {
    pub language: Language,
}

impl FromStr for LanguageConfig {
    type Err = String;
    fn from_str(it: &str) -> Result<LanguageConfig, Self::Err> {
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

    pub fn to_rust_lang(&self) -> RustLang {
        match self.language {
            Language::EN => RustLang::EN,
            Language::FR => RustLang::FR,
            Language::DE => RustLang::DE,
            Language::KO => RustLang::KO,
            Language::ES => RustLang::ES,
        }
    }
}
