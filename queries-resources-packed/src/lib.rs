#[macro_use]
extern crate error_chain;
extern crate phf;
extern crate nlu_utils;

mod errors {
    error_chain! {
    }
}

use errors::*;
pub use errors::Error;
use nlu_utils::language::Language;

include!(concat!(env!("OUT_DIR"), "/phf.rs"));

pub fn stem(language: Language, word: &str) -> Result<String> {
    if let Some(stem) = match language {
        Language::EN => &STEMS_EN,
        Language::FR => &STEMS_FR,
        Language::ES => &STEMS_ES,
        Language::DE => &STEMS_DE,
        _ => bail!("stem not supported for {}", language.to_string()),
    }
        .get(word) {
        Ok(stem.to_string())
    } else {
        Ok(word.to_string())
    }
}

pub fn word_cluster(cluster_name: &str, language: Language, word: &str) -> Result<Option<String>> {
    match language {
        Language::EN => match cluster_name {
            "brown_clusters" => Ok(WORD_CLUSTERS_EN_BROWN_CLUSTERS.get(word).map(|c| c.to_string())),
            _ => bail!("word cluster '{}' not supported for language {}", cluster_name, language.to_string())
        },
        _ => bail!("brown clusters not supported for {} language", language.to_string())
    }
}

pub fn gazetteer_hits(language: Language, gazetteer_name: &str, word: &str) -> Result<bool> {
    Ok(match language {
        Language::EN => match gazetteer_name {
            "top_10000_nouns" => &GAZETTEER_EN_TOP_10000_NOUNS,
            "cities_us" => &GAZETTEER_EN_CITIES_US,
            "cities_world" => &GAZETTEER_EN_CITIES_WORLD,
            "countries" => &GAZETTEER_EN_COUNTRIES,
            "states_us" => &GAZETTEER_EN_STATES_US,
            "stop_words" => &GAZETTEER_EN_STOP_WORDS,
            "street_identifier" => &GAZETTEER_EN_STREET_IDENTIFIER,
            "top_10000_words" => &GAZETTEER_EN_TOP_10000_WORDS,
            "top_10000_nouns_stem" => &GAZETTEER_EN_TOP_10000_NOUNS_STEM,
            "cities_us_stem" => &GAZETTEER_EN_CITIES_US_STEM,
            "cities_world_stem" => &GAZETTEER_EN_CITIES_WORLD_STEM,
            "countries_stem" => &GAZETTEER_EN_COUNTRIES_STEM,
            "states_us_stem" => &GAZETTEER_EN_STATES_US_STEM,
            "stop_words_stem" => &GAZETTEER_EN_STOP_WORDS_STEM,
            "street_identifier_stem" => &GAZETTEER_EN_STREET_IDENTIFIER_STEM,
            "top_10000_words_stem" => &GAZETTEER_EN_TOP_10000_WORDS_STEM,
            _ => bail!("gazetteer {} not supported for language {}", gazetteer_name, language.to_string())
        },
        Language::FR => match gazetteer_name {
            "cities_france" => &GAZETTEER_FR_CITIES_FRANCE,
            "cities_world" => &GAZETTEER_FR_CITIES_WORLD,
            "countries" => &GAZETTEER_FR_COUNTRIES,
            "departements_france" => &GAZETTEER_FR_DEPARTEMENTS_FRANCE,
            "regions_france" => &GAZETTEER_FR_REGIONS_FRANCE,
            "stop_words" => &GAZETTEER_FR_STOP_WORDS,
            "street_identifier" => &GAZETTEER_FR_STREET_IDENTIFIER,
            "top_10000_words" => &GAZETTEER_FR_TOP_10000_WORDS,
            "cities_france_stem" => &GAZETTEER_FR_CITIES_FRANCE_STEM,
            "cities_world_stem" => &GAZETTEER_FR_CITIES_WORLD_STEM,
            "countries_stem" => &GAZETTEER_FR_COUNTRIES_STEM,
            "departements_france_stem" => &GAZETTEER_FR_DEPARTEMENTS_FRANCE_STEM,
            "regions_france_stem" => &GAZETTEER_FR_REGIONS_FRANCE_STEM,
            "stop_words_stem" => &GAZETTEER_FR_STOP_WORDS_STEM,
            "street_identifier_stem" => &GAZETTEER_FR_STREET_IDENTIFIER_STEM,
            "top_10000_words_stem" => &GAZETTEER_FR_TOP_10000_WORDS_STEM,
            _ => bail!("gazetteer {} not supported for language {}", gazetteer_name, language.to_string())
        },
        Language::DE => match gazetteer_name {
            "cities_germany" => &GAZETTEER_DE_CITIES_GERMANY,
            "cities_world" => &GAZETTEER_DE_CITIES_WORLD,
            "countries" => &GAZETTEER_DE_COUNTRIES,
            "lander_germany" => &GAZETTEER_DE_LANDER_GERMANY,
            "stop_words" => &GAZETTEER_DE_STOP_WORDS,
            "street_identifier" => &GAZETTEER_DE_STREET_IDENTIFIER,
            "cities_germany_stem" => &GAZETTEER_DE_CITIES_GERMANY_STEM,
            "cities_world_stem" => &GAZETTEER_DE_CITIES_WORLD_STEM,
            "countries_stem" => &GAZETTEER_DE_COUNTRIES_STEM,
            "lander_germany_stem" => &GAZETTEER_DE_LANDER_GERMANY_STEM,
            "stop_words_stem" => &GAZETTEER_DE_STOP_WORDS_STEM,
            "street_identifier_stem" => &GAZETTEER_DE_STREET_IDENTIFIER_STEM,
            _ => bail!("gazetteer {} not supported for language {}", gazetteer_name, language.to_string())
        },
        _ => bail!("no gazetteers supported for {} language", language.to_string())
    }.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn stem_works() {
        assert_eq!(stem(Language::from_str("en"), "billing").unwrap(), "bill")
    }

    #[test]
    fn brown_clusters_works() {
        assert_eq!(word_cluster("brown_clusters", Language::from_str("en"), "groovy").unwrap().unwrap(), "11111000111111")
    }

    #[test]
    fn gazetteers_works() {
        assert_eq!(gazetteer_hits(Language::from_str("en"), "top_10000_words", "car").unwrap(), true);
        assert_eq!(gazetteer_hits(Language::from_str("en"), "top_10000_words", "qsmldkfjdk").unwrap(), false)
    }
}
