use std::collections::HashMap;
use std::io::Read;

use csv;
use errors::*;
use snips_nlu_ontology::Language;

pub trait WordClusterer: Send + Sync {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

pub struct HashMapWordClusterer {
    values: HashMap<String, String>
}

impl HashMapWordClusterer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .from_reader(reader);
        let mut values = HashMap::<String, String>::new();
        for record in csv_reader.records() {
            let elements = record?;
            values.insert(elements[0].to_string(), elements[1].to_string());
        }

        Ok(Self { values })
    }
}

impl WordClusterer for HashMapWordClusterer {
    fn get_cluster(&self, word: &str) -> Option<String> {
        self.values
            .get(word)
            .map(|v| v.to_string())
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
    fn hashmap_word_clusterer_works() {
        // Given
        let clusters: &[u8] = r#"
hello	1111111111111
world	1111110111111
"yolo	1111100111111
"#.as_ref();

        // When
        let clusterer = HashMapWordClusterer::from_reader(clusters);

        // Then
        assert!(clusterer.is_ok());
        let clusterer = clusterer.unwrap();
        assert_eq!(clusterer.get_cluster("hello"), Some("1111111111111".to_string()));
        assert_eq!(clusterer.get_cluster("world"), Some("1111110111111".to_string()));
        assert_eq!(clusterer.get_cluster("\"yolo"), Some("1111100111111".to_string()));
        assert_eq!(clusterer.get_cluster("unknown"), None);
    }
}
