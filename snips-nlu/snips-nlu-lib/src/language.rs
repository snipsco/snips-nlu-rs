use std::result::Result;
use std::str::FromStr;

use snips_nlu_ontology::Language;
use nlu_utils::language::Language as NluUtilsLanguage;

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
}

pub trait FromLanguage {
    fn from_language(Language) -> Self;
}

impl FromLanguage for NluUtilsLanguage{
    fn from_language(l: Language) -> Self {
        match l {
            Language::DE => NluUtilsLanguage::DE,
            Language::EN => NluUtilsLanguage::EN,
            Language::ES => NluUtilsLanguage::ES,
            Language::FR => NluUtilsLanguage::FR,
            Language::KO => NluUtilsLanguage::KO,
        }
    }
}
