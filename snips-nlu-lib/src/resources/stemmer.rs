use std::collections::HashMap;
use std::io::Read;
use std::iter::FromIterator;

use csv;
use errors::*;

pub trait Stemmer: Send + Sync {
    fn stem(&self, value: &str) -> String;
}

pub struct HashMapStemmer {
    values: HashMap<String, String>
}

impl HashMapStemmer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut values = HashMap::<String, String>::new();
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
                values.insert(value.to_string(), stem.to_string());
            }
        }
        Ok(Self { values })
    }
}

impl<I> From<I> for HashMapStemmer where I: Iterator<Item=(String, String)> {
    fn from(values_it: I) -> Self {
        Self {
            values: HashMap::from_iter(values_it),
        }
    }
}

impl Stemmer for HashMapStemmer {
    fn stem(&self, value: &str) -> String {
        self.values
            .get(value)
            .map(|v| v.to_string())
            .unwrap_or_else(|| value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_stemmer_works() {
        // Given
        let stems: &[u8] = r#"
investigate,investigated,investigation,"investigate
do,done,don't,doing,did,does"#.as_ref();

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
