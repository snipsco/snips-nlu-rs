use crate::errors::*;
use snips_nlu_utils::string::{hash_str_to_i32, normalize};
use std::collections::HashMap;
use std::io::Read;
use std::iter::FromIterator;

pub trait Stemmer: Send + Sync {
    fn stem(&self, value: &str) -> String;
}

pub struct HashMapStemmer {
    values: HashMap<i32, String>,
}

impl HashMapStemmer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut values = HashMap::new();
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .quoting(false)
            .flexible(true)
            .has_headers(false)
            .from_reader(reader);

        for record in csv_reader.records() {
            let elements = record?;
            let stem = &elements[0];
            for value in elements.iter().skip(1) {
                values.insert(hash_str_to_i32(value), stem.to_string());
            }
        }
        Ok(Self { values })
    }
}

impl FromIterator<(String, String)> for HashMapStemmer {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            values: iter
                .into_iter()
                .map(|(str_key, str_value)| (hash_str_to_i32(&*str_key), str_value))
                .collect(),
        }
    }
}

impl Stemmer for HashMapStemmer {
    fn stem(&self, value: &str) -> String {
        self.values
            .get(&hash_str_to_i32(&*normalize(value)))
            .map(|v| v.to_string())
            .unwrap_or_else(|| value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_stemmer() {
        // Given
        let stems: &[u8] = r#"
investigate,investigated,investigation,"investigate
do,done,don't,doing,did,does"#
            .as_ref();

        // When
        let stemmer = HashMapStemmer::from_reader(stems);

        // Then
        assert!(stemmer.is_ok());
        let stemmer = stemmer.unwrap();
        assert_eq!(stemmer.stem("don't"), "do".to_string());
        assert_eq!(stemmer.stem("does"), "do".to_string());
        assert_eq!(stemmer.stem("\"investigate"), "investigate".to_string());
        assert_eq!(stemmer.stem("unknown"), "unknown".to_string());
    }
}
