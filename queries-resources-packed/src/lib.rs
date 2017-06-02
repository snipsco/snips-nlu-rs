#[macro_use]
extern crate error_chain;
extern crate phf;

mod errors {
    error_chain! {

    }
}

use errors::*;
pub use errors::Error;

include!(concat!(env!("OUT_DIR"), "/phf.rs"));

pub fn stem(language: &str, word: &str) -> Result<String> {
    if let Some(stem) = match language {
        "en" => &STEMS_EN,
        "fr" => &STEMS_FR,
        "es" => &STEMS_ES,
        _ => bail!("stem not supported for {}", language),
    }
        .get(word) {
        Ok(stem.to_string())
    } else {
        Ok(word.to_string())
    }
}

pub fn word_cluster(cluster_name: &str, language: &str, word: &str) -> Result<Option<String>> {
    match language {
        "en" => match cluster_name {
            "brown_clusters" => Ok(WORD_CLUSTERS_EN_BROWN_CLUSTERS.get(word).map(|c| c.to_string())),
            _ => bail!("word cluster '{}' not supported for language {}", cluster_name, language)
        },
        _ => bail!("brown clusters not supported for {} language", language)
    }
}

pub fn gazetteer_hits(language: &str, gazetteer_name: &str, word: &str) -> Result<bool> {
    Ok(match language {
        "en" => match gazetteer_name {
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
            _ => bail!("gazetteer {} not supported for language {}", gazetteer_name, language)
        },
        _ => bail!("no gazetteers supported for {} language", language)
    }.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_works() {
        assert_eq!(stem("en", "billing").unwrap(), "bill")
    }

    #[test]
    fn brown_clusters_works() {
        assert_eq!(word_cluster("brown_clusters", "en", "groovy").unwrap().unwrap(), "11111000111111")
    }

    #[test]
    fn gazetteers_works() {
        assert_eq!(gazetteer_hits("en", "top_10000_words", "car").unwrap(), true);
        assert_eq!(gazetteer_hits("en", "top_10000_words", "qsmldkfjdk").unwrap(), false)
    }
}
