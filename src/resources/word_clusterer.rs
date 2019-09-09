use crate::errors::*;
use itertools::Either;
use snips_nlu_ontology::Language;
use snips_nlu_utils::string::hash_str_to_i32;
use std::collections::HashMap;
use std::io::Read;
use std::str::FromStr;

pub trait WordClusterer: Send + Sync {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct HashMapWordClusterer {
    // This implementation allows to support i32 representation for word clusters
    // in a backward compatible manner
    values: Either<HashMap<i32, u16>, HashMap<i32, String>>,
}

impl HashMapWordClusterer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .from_reader(reader);
        // This flag is switched to false as soon as a record is found which cannot
        // be converted to a u16
        let mut u16_casting_ok = true;
        let mut u16_values = HashMap::<i32, u16>::new();
        let mut str_values = HashMap::<i32, String>::new();
        for record in csv_reader.records() {
            let elements = record?;
            let hashed_key = hash_str_to_i32(elements[0].as_ref());
            // Casting into u16 is attempted only when all previous clusters were converted
            // successfully
            if u16_casting_ok {
                match u16::from_str(elements[1].as_ref()) {
                    Ok(u16_value) => {
                        u16_values.insert(hashed_key, u16_value);
                    }
                    Err(_) => {
                        // A word cluster cannot be converted into a u16, let's move all the
                        // previously stored clusters into a raw string representation
                        for (hash, value) in u16_values.iter() {
                            str_values.insert(*hash, format!("{}", value));
                        }
                        str_values.insert(hashed_key, elements[1].to_string());
                        u16_casting_ok = false;
                        u16_values.clear();
                    }
                }
            } else {
                str_values.insert(hashed_key, elements[1].to_string());
            }
        }
        Ok(Self {
            values: if u16_casting_ok {
                Either::Left(u16_values)
            } else {
                Either::Right(str_values)
            },
        })
    }
}

impl WordClusterer for HashMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        let hashed_key = hash_str_to_i32(word);
        match &self.values {
            Either::Left(u16_values) => u16_values.get(&hashed_key).map(|v| format!("{}", v)),
            Either::Right(str_values) => str_values.get(&hashed_key).cloned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordClustererConfiguration {
    language: Language,
    clusters_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_word_clusterer() {
        // Given
        let clusters: &[u8] = r#"
hello	42
world	123
"yolo	cluster_which_is_not_u16
"#
        .as_ref();

        // When
        let clusterer = HashMapWordClusterer::from_reader(clusters);

        // Then
        assert!(clusterer.is_ok());
        let clusterer = clusterer.unwrap();
        assert_eq!(clusterer.get_cluster("hello"), Some("42".to_string()));
        assert_eq!(clusterer.get_cluster("world"), Some("123".to_string()));
        assert_eq!(clusterer.get_cluster("\"yolo"), Some("cluster_which_is_not_u16".to_string()));
        assert_eq!(clusterer.get_cluster("unknown"), None);
    }
}
