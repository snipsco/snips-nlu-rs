use crate::errors::*;
use snips_nlu_ontology::Language;
use snips_nlu_utils::string::hash_str_to_i32;
use std::collections::HashMap;
use std::io::Read;
use std::str::FromStr;

pub trait WordClusterer: Send + Sync {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct HashMapWordClusterer {
    values: HashMap<i32, u16>,
}

impl HashMapWordClusterer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .from_reader(reader);
        let mut values = HashMap::<i32, u16>::new();
        for record in csv_reader.records() {
            let elements = record?;
            values.insert(
                hash_str_to_i32(elements[0].as_ref()),
                u16::from_str(elements[1].as_ref())?,
            );
        }

        Ok(Self { values })
    }
}

impl WordClusterer for HashMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        self.values
            .get(&hash_str_to_i32(word))
            .map(|v| format!("{}", v))
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
"yolo	5960
"#
        .as_ref();

        // When
        let clusterer = HashMapWordClusterer::from_reader(clusters);

        // Then
        assert!(clusterer.is_ok());
        let clusterer = clusterer.unwrap();
        assert_eq!(clusterer.get_cluster("hello"), Some("42".to_string()));
        assert_eq!(clusterer.get_cluster("world"), Some("123".to_string()));
        assert_eq!(clusterer.get_cluster("\"yolo"), Some("5960".to_string()));
        assert_eq!(clusterer.get_cluster("unknown"), None);
    }
}
