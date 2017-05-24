extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate queries_preprocessor;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Csv(::csv::Error);
        }
    }
}

pub use errors::Error;

pub mod stems {
    use std::collections::HashMap;
    use std::io::Read;

    use csv;
    use itertools::Itertools;

    use errors::*;

    fn parse_lexemes<R: Read>(lexemes_file_reader: R) -> Result<HashMap<String, String>> {
        let mut csv_reader = csv::Reader::from_reader(lexemes_file_reader)
            .delimiter(b';')
            .has_headers(false);

        let mut result = HashMap::new();

        for row in csv_reader.decode() {
            let (value, keys): (String, String) = row?;


            keys.split(",")
                .foreach(|key| { result.insert(key.to_string(), value.clone()); });
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

            result.insert(key, value);
        }
        Ok(result)
    }

    pub fn en() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(&include_bytes!("../snips-nlu-resources/en/top_10000_words_inflected.txt")[..])?;
        result.extend(parse_lexemes(&include_bytes!("../snips-nlu-resources/en/top_1000_verbs_lexemes.txt")[..])?);
        Ok(result)
    }

    pub fn fr() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(&include_bytes!("../snips-nlu-resources/fr/top_10000_words_inflected.txt")[..])?;
        result.extend(parse_lexemes(&include_bytes!("../snips-nlu-resources/fr/top_2000_verbs_lexemes.txt")[..])?);
        Ok(result)
    }

    pub fn es() -> Result<HashMap<String, String>> {
        let mut result = parse_inflections(&include_bytes!("../snips-nlu-resources/es/top_10000_words_inflected.txt")[..])?;
        result.extend(parse_lexemes(&include_bytes!("../snips-nlu-resources/es/top_1000_verbs_lexemes.txt")[..])?);
        Ok(result)
    }
}


pub mod word_clusters {
    use std::collections::HashMap;
    use std::io::Read;

    use csv;

    use errors::*;

    fn parse_clusters<R: Read>(clusters_file_reader: R) -> Result<HashMap<String, String>> {
        let mut csv_reader = csv::Reader::from_reader(clusters_file_reader)
            .delimiter(b'\t')
            .has_headers(false);

        let mut result = HashMap::new();

        for row in csv_reader.decode() {
            let (key, value): (String, String) = row?;

            result.insert(key, value);
        }
        Ok(result)
    }


    pub mod en {
        use std::collections::HashMap;

        use errors::*;

        pub fn brown_clusters() -> Result<HashMap<String, String>> {
            super::parse_clusters(&include_bytes!("../snips-nlu-resources/en/brown_clusters.txt")[..])
        }
    }
}

pub mod gazetteer {
    use std::collections::HashSet;
    use std::io::{BufRead, BufReader, Read};

    use itertools::Itertools;

    use errors::*;
    use queries_preprocessor::tokenize;


    fn parse_gazetteer<R: Read, F>(gazetteer_reader: R, stem_fn: F) -> Result<HashSet<String>>
        where F: Fn(String) -> String {
        let reader = BufReader::new(gazetteer_reader);
        let mut result = HashSet::new();

        for line in reader.lines() {
            let normalized = line?.trim().to_lowercase();
            if !normalized.is_empty() {
                let tokens = tokenize(&normalized);
                result.insert(tokens.into_iter().map(|t| stem_fn(t.value)).join(" "));
            }
        }
        Ok(result)
    }

    pub mod en {
        use std::collections::{HashMap, HashSet};
        use errors::*;
        use stems;

        fn stem_en(input: String) -> String {
            lazy_static! {
                static ref STEMS_EN: HashMap<String, String> = stems::en().unwrap();
            }
            STEMS_EN.get(&input).unwrap_or(&input).to_string()
        }

        fn no_stem(input: String) -> String {
            input
        }

        macro_rules! create_gazetteer {
            ($gazetteer_name:ident) => {
                pub fn $gazetteer_name() -> Result<HashSet<String>> {
                    super::parse_gazetteer(&include_bytes!(concat!("../snips-nlu-resources/en/", stringify!($gazetteer_name), ".txt"))[..],
                                           no_stem)
                }
            };
            ($function_name:ident, $gazetteer_name:ident, $stem:ident) => {
                pub fn $function_name() -> Result<HashSet<String>> {
                    super::parse_gazetteer(&include_bytes!(concat!("../snips-nlu-resources/en/", stringify!($gazetteer_name), ".txt"))[..],
                                           $stem)
                }
            };
        }

        create_gazetteer!(top_10000_nouns);
        create_gazetteer!(cities_us);
        create_gazetteer!(cities_world);
        create_gazetteer!(countries);
        create_gazetteer!(states_us);
        create_gazetteer!(stop_words);
        create_gazetteer!(street_identifier);
        create_gazetteer!(top_10000_words);

        create_gazetteer!(top_10000_nouns_stem, top_10000_nouns, stem_en);
        create_gazetteer!(cities_us_stem, cities_us, stem_en);
        create_gazetteer!(cities_world_stem, cities_world, stem_en);
        create_gazetteer!(countries_stem, countries, stem_en);
        create_gazetteer!(states_us_stem, states_us, stem_en);
        create_gazetteer!(stop_words_stem, stop_words, stem_en);
        create_gazetteer!(street_identifier_stem, street_identifier, stem_en);
        create_gazetteer!(top_10000_words_stem, top_10000_words, stem_en);
    }
}
