use nlu_utils::language::Language as NluUtilsLanguage;
use snips_nlu_ontology::Language;

pub trait FromLanguage {
    fn from_language(l: Language) -> Self;
}

impl FromLanguage for NluUtilsLanguage {
    fn from_language(l: Language) -> Self {
        match l {
            Language::DE => NluUtilsLanguage::DE,
            Language::EN => NluUtilsLanguage::EN,
            Language::ES => NluUtilsLanguage::ES,
            Language::FR => NluUtilsLanguage::FR,
            Language::KO => NluUtilsLanguage::KO,
            Language::JA => NluUtilsLanguage::JA,
        }
    }
}
