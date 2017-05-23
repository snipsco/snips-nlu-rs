#[macro_use]
extern crate error_chain;
extern crate phf;

mod errors {
    error_chain! {

    }
}

use errors::*;

include!(concat!(env!("OUT_DIR"), "/phf.rs"));

pub fn stem(language: &str, word: &str) -> Result<String> {
    if let Some(stem) = match language {
        "en" => &STEMS_EN,
        "fn" => &STEMS_FR,
        "es" => &STEMS_ES,
        _ => bail!("stem not supported for {}", language),
    }
        .get(word) {
        Ok(stem.to_string())
    } else {
        Ok(word.to_string())
    }
}

pub fn brown_clusters(language: &str, word: &str) -> Result<Option<String>> {
    match language {
        "en" => Ok(WORD_CLUSTERS_EN_BROWN_CLUSTERS.get(word).map(|c| c.to_string())),
        _ => bail!("brown clusters not supported for {} language")
    }
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
        assert_eq!(brown_clusters("en", "groovy").unwrap().unwrap(), "11111000111111")
    }
}
