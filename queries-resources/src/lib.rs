extern crate csv;

#[macro_use]
extern crate error_chain;
extern crate itertools;


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
