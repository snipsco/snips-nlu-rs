extern crate csv;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate snips_nlu_utils as nlu_utils;

pub mod errors {
    pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
}

pub mod stems {
    use std::collections::HashMap;
    use std::io::Read;

    use csv;
    use itertools::Itertools;

    use nlu_utils::string::normalize;

    use errors::*;

    pub fn no_stem(input: String) -> String {
        input
    }

    fn parse_lexemes<R: Read>(lexemes_file_reader: R) -> Result<HashMap<String, String>> {
        let mut csv_reader = csv::Reader::from_reader(lexemes_file_reader)
            .delimiter(b';')
            .has_headers(false);

        let mut result = HashMap::new();

        for row in csv_reader.decode() {
            let (value, keys): (String, String) = row?;
            let normalized_value = normalize(&value);
            keys.split(",").foreach(|key| {
                result.insert(normalize(key).to_string(), normalized_value.clone());
            });
        }
        Ok(result)
    }

    fn parse_inflections<R: Read>(inflections_file_reader: R) -> Result<HashMap<String, String>> {
        let mut csv_reader = csv::Reader::from_reader(inflections_file_reader)
            .delimiter(b';')
            .has_headers(false);

        let mut result = HashMap::new();

        for row in csv_reader.decode() {
            let (key, value): (String, String) = row?;

            result.insert(normalize(&key), normalize(&value));
        }
        Ok(result)
    }

    pub fn en() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(
            &include_bytes!("../snips-nlu-resources/en/top_10000_words_inflected.txt")[..],
        )?;
        result.extend(parse_lexemes(
            &include_bytes!("../snips-nlu-resources/en/top_1000_verbs_lexemes.txt")[..],
        )?);
        Ok(result)
    }

    pub fn fr() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(
            &include_bytes!("../snips-nlu-resources/fr/top_10000_words_inflected.txt")[..],
        )?;
        result.extend(parse_lexemes(
            &include_bytes!("../snips-nlu-resources/fr/top_2000_verbs_lexemes.txt")[..],
        )?);
        Ok(result)
    }

    pub fn es() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(
            &include_bytes!("../snips-nlu-resources/es/top_10000_words_inflected.txt")[..],
        )?;
        result.extend(parse_lexemes(
            &include_bytes!("../snips-nlu-resources/es/top_1000_verbs_lexemes.txt")[..],
        )?);
        Ok(result)
    }

    pub fn de() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(
            &include_bytes!("../snips-nlu-resources/de/top_10000_words_inflected.txt")[..],
        )?;
        result.extend(parse_lexemes(
            &include_bytes!("../snips-nlu-resources/de/top_1000_verbs_lexemes.txt")[..],
        )?);
        Ok(result)
    }
}

pub mod word_clusters {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Read};

    use errors::*;

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

    pub mod en {
        use std::collections::HashMap;

        use errors::*;

        pub fn brown_clusters() -> Result<HashMap<String, String>> {
            super::parse_clusters(
                &include_bytes!("../snips-nlu-resources/en/brown_clusters.txt")[..],
            )
        }
    }
}

pub mod gazetteer {
    use std::collections::HashSet;
    use std::io::{BufRead, BufReader, Read};

    use itertools::Itertools;

    use errors::*;
    use nlu_utils::language::Language;
    use nlu_utils::string::normalize;
    use nlu_utils::token::tokenize_light;

    fn parse_gazetteer<R: Read, F>(
        gazetteer_reader: R,
        stem_fn: F,
        language: Language,
    ) -> Result<HashSet<String>>
    where
        F: Fn(String) -> String,
    {
        let reader = BufReader::new(gazetteer_reader);
        let mut result = HashSet::new();

        for line in reader.lines() {
            let normalized = normalize(&line?);
            if !normalized.is_empty() {
                let tokens = tokenize_light(&normalized, language);
                result.insert(
                    tokens
                        .into_iter()
                        .map(|t| stem_fn(t))
                        .join(language.default_separator()),
                );
            }
        }
        Ok(result)
    }

    macro_rules! create_gazetteer {
        ($language:ident, $gazetteer_name:ident) => {
            pub fn $gazetteer_name() -> Result<HashSet<String>> {
                super::parse_gazetteer(
                    &include_bytes!(concat!(
                        "../snips-nlu-resources/",
                        stringify!($language),
                        "/",
                        stringify!($gazetteer_name),
                        ".txt"
                    ))[..],
                    stems::no_stem,
                    Language::from_str(stringify!($language)).map_err(::failure::err_msg)?,
                )
            }
        };
        ($language:ident, $function_name:ident, $gazetteer_name:ident, $stem:ident) => {
            pub fn $function_name() -> Result<HashSet<String>> {
                super::parse_gazetteer(
                    &include_bytes!(concat!(
                        "../snips-nlu-resources/",
                        stringify!($language),
                        "/",
                        stringify!($gazetteer_name),
                        ".txt"
                    ))[..],
                    $stem,
                    Language::from_str(stringify!($language)).map_err(::failure::err_msg)?,
                )
            }
        };
    }

    pub mod en {
        use errors::*;
        use std::collections::{HashMap, HashSet};
        use std::str::FromStr;
        use stems;

        use nlu_utils::language::Language;

        fn stem_en(input: String) -> String {
            lazy_static! {
                static ref STEMS_EN: HashMap<String, String> = stems::en().unwrap();
            }
            STEMS_EN.get(&input).unwrap_or(&input).to_string()
        }

        create_gazetteer!(en, stop_words);
        create_gazetteer!(en, top_10000_nouns);
        create_gazetteer!(en, top_10000_words);

        create_gazetteer!(en, stop_words_stem, stop_words, stem_en);
        create_gazetteer!(en, top_10000_nouns_stem, top_10000_nouns, stem_en);
        create_gazetteer!(en, top_10000_words_stem, top_10000_words, stem_en);
    }

    pub mod fr {
        use errors::*;
        use std::collections::{HashMap, HashSet};
        use std::str::FromStr;
        use stems;

        use nlu_utils::language::Language;

        fn stem_fr(input: String) -> String {
            lazy_static! {
                static ref STEMS_FR: HashMap<String, String> = stems::fr().unwrap();
            }
            STEMS_FR.get(&input).unwrap_or(&input).to_string()
        }

        create_gazetteer!(fr, stop_words);
        create_gazetteer!(fr, top_10000_words);

        create_gazetteer!(fr, stop_words_stem, stop_words, stem_fr);
        create_gazetteer!(fr, top_10000_words_stem, top_10000_words, stem_fr);
    }

    pub mod de {
        use errors::*;
        use std::collections::{HashMap, HashSet};
        use std::str::FromStr;
        use stems;

        use nlu_utils::language::Language;

        fn stem_de(input: String) -> String {
            lazy_static! {
                static ref STEMS_DE: HashMap<String, String> = stems::de().unwrap();
            }
            STEMS_DE.get(&input).unwrap_or(&input).to_string()
        }

        create_gazetteer!(de, stop_words);
        create_gazetteer!(de, top_10000_words);

        create_gazetteer!(de, stop_words_stem, stop_words, stem_de);
        create_gazetteer!(de, top_10000_words_stem, top_10000_words, stem_de);
    }

    pub mod es {
        use errors::*;
        use std::collections::{HashMap, HashSet};
        use std::str::FromStr;
        use stems;

        use nlu_utils::language::Language;

        fn stem_es(input: String) -> String {
            lazy_static! {
                static ref STEMS_ES: HashMap<String, String> = stems::es().unwrap();
            }
            STEMS_ES.get(&input).unwrap_or(&input).to_string()
        }

        create_gazetteer!(es, stop_words);
        create_gazetteer!(es, top_10000_words);

        create_gazetteer!(es, stop_words_stem, stop_words, stem_es);
        create_gazetteer!(es, top_10000_words_stem, top_10000_words, stem_es);
    }
}
