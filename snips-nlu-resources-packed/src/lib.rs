#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate phf;
extern crate snips_nlu_ontology;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use snips_nlu_ontology::Language;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

include!(concat!(env!("OUT_DIR"), "/phf.rs"));

fn parse_clusters<R: Read>(clusters_file_reader: R) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    let f = BufReader::new(clusters_file_reader);
    for (i, row) in f.lines().enumerate() {
        let line = row?;
        let split: Vec<&str> = line.split("\t").collect();
        if split.len() == 2 {
            result.insert(split[0].to_string(), split[1].to_string());
        } else {
            Err(format_err!("Invalid line at index {:?}", i))?;
        }
    }
    Ok(result)
}

lazy_static!{
    static ref WORD_CLUSTERS_JA_W2V_CLUSTERS: HashMap<String, String> = parse_clusters(&include_bytes!("../../snips-nlu-resources/snips-nlu-resources/ja/w2v_clusters.txt")[..]).unwrap();
}

pub fn stem(language: Language, word: &str) -> Result<String> {
    if let Some(stem) = match language {
        Language::EN => &STEMS_EN,
        Language::FR => &STEMS_FR,
        Language::ES => &STEMS_ES,
        Language::DE => &STEMS_DE,
        _ => bail!("stem not supported for {}", language.to_string()),
    }.get(word)
    {
        Ok(stem.to_string())
    } else {
        Ok(word.to_string())
    }
}

pub fn word_cluster(cluster_name: &str, language: Language, word: &str) -> Result<Option<String>> {
    match language {
        Language::EN => match cluster_name {
            "brown_clusters" => Ok(WORD_CLUSTERS_EN_BROWN_CLUSTERS
                .get(word)
                .map(|c| c.to_string())),
            _ => bail!(
                "word cluster '{}' not supported for language {}",
                cluster_name,
                language.to_string()
            ),
        },
        Language::JA => match cluster_name {
            "w2v_clusters" => Ok(WORD_CLUSTERS_JA_W2V_CLUSTERS
                .get(word)
                .map(|c| c.to_string())),
            _ => bail!(
                "word cluster '{}' not supported for language {}",
                cluster_name,
                language.to_string()
            ),
        },
        _ => bail!(
            "brown clusters not supported for {} language",
            language.to_string()
        ),
    }
}

pub fn gazetteer_hits(language: Language, gazetteer_name: &str, word: &str) -> Result<bool> {
    Ok(match language {
        Language::DE => match gazetteer_name {
            "stop_words" => &GAZETTEER_DE_STOP_WORDS,
            "stop_words_stem" => &GAZETTEER_DE_STOP_WORDS_STEM,
            "top_10000_words" => &GAZETTEER_DE_TOP_10000_WORDS,
            "top_10000_words_stem" => &GAZETTEER_DE_TOP_10000_WORDS_STEM,
            _ => bail!(
                "gazetteer {} not supported for language {}",
                gazetteer_name,
                language.to_string()
            ),
        },
        Language::EN => match gazetteer_name {
            "stop_words" => &GAZETTEER_EN_STOP_WORDS,
            "stop_words_stem" => &GAZETTEER_EN_STOP_WORDS_STEM,
            "top_10000_nouns" => &GAZETTEER_EN_TOP_10000_NOUNS,
            "top_10000_nouns_stem" => &GAZETTEER_EN_TOP_10000_NOUNS_STEM,
            "top_10000_words" => &GAZETTEER_EN_TOP_10000_WORDS,
            "top_10000_words_stem" => &GAZETTEER_EN_TOP_10000_WORDS_STEM,
            _ => bail!(
                "gazetteer {} not supported for language {}",
                gazetteer_name,
                language.to_string()
            ),
        },
        Language::ES => match gazetteer_name {
            "stop_words" => &GAZETTEER_ES_STOP_WORDS,
            "stop_words_stem" => &GAZETTEER_ES_STOP_WORDS_STEM,
            "top_10000_words" => &GAZETTEER_ES_TOP_10000_WORDS,
            "top_10000_words_stem" => &GAZETTEER_ES_TOP_10000_WORDS_STEM,
            _ => bail!(
                "gazetteer {} not supported for language {}",
                gazetteer_name,
                language.to_string()
            ),
        },
        Language::FR => match gazetteer_name {
            "stop_words" => &GAZETTEER_FR_STOP_WORDS,
            "stop_words_stem" => &GAZETTEER_FR_STOP_WORDS_STEM,
            "top_10000_words" => &GAZETTEER_FR_TOP_10000_WORDS,
            "top_10000_words_stem" => &GAZETTEER_FR_TOP_10000_WORDS_STEM,
            _ => bail!(
                "gazetteer {} not supported for language {}",
                gazetteer_name,
                language.to_string()
            ),
        },
        _ => bail!(
            "no gazetteers supported for {} language",
            language.to_string()
        ),
    }.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn stem_works() {
        assert_eq!(
            stem(Language::from_str("en").unwrap(), "billing").unwrap(),
            "bill"
        )
    }

    #[test]
    fn brown_clusters_works() {
        assert_eq!(
            word_cluster(
                "brown_clusters",
                Language::from_str("en").unwrap(),
                "groovy"
            ).unwrap()
                .unwrap(),
            "11111000111111"
        )
    }

    #[test]
    fn ja_w2v_clusters_works() {
        assert_eq!(
            word_cluster(
                "w2v_clusters",
                Language::from_str("ja").unwrap(),
                "スメラ"
            ).unwrap()
                .unwrap(),
            "0"
        )
    }

    #[test]
    fn gazetteers_works() {
        assert_eq!(
            gazetteer_hits(Language::from_str("en").unwrap(), "top_10000_words", "car").unwrap(),
            true
        );
        assert_eq!(
            gazetteer_hits(
                Language::from_str("en").unwrap(),
                "top_10000_words",
                "qsmldkfjdk"
            ).unwrap(),
            false
        )
    }
}
