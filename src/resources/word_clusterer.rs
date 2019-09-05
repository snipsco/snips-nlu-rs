use crate::errors::*;
use failure::ResultExt;
use snips_nlu_ontology::Language;
use snips_nlu_utils::string::hash_str_to_i32;
use std::collections::HashMap;
use std::io::Read;

pub trait WordClusterer: Send + Sync {
    fn get_cluster(&self, word: &str) -> Option<String>;
}

/// Handles hierarchical word clusters represented with a binary path like '00100110'
/// The maximal size of the path is 16, hence each cluster is represented with a u16 number
/// which corresponds to the binary path with trailing zeros
pub struct HierarchicalWordClusterer {
    values: HashMap<i32, u16>,
}

impl HierarchicalWordClusterer {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .from_reader(reader);
        let mut values = HashMap::<i32, u16>::new();
        for record in csv_reader.records() {
            let elements = record?;
            let cluster_str: &str = elements[1].as_ref();
            // Trailing zeros are added to ensure that each cluster representation is exactly
            // 16 bits long
            let trailing_zeros: String = std::iter::repeat("0")
                .take(16 - cluster_str.len())
                .collect();
            let full_cluster = format!("{}{}", cluster_str, trailing_zeros);
            let cluster = u16::from_str_radix(&full_cluster, 2).with_context(|_| {
                format!(
                    "Word cluster is not a binary digit: {}",
                    elements[1].as_ref() as &str
                )
            })?;
            values.insert(hash_str_to_i32(elements[0].as_ref()), cluster);
        }

        Ok(Self { values })
    }
}

impl WordClusterer for HierarchicalWordClusterer {
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
hello	000000000000011
world	0000000000010
"yolo	0000000000011
"#
        .as_ref();

        // When
        let clusterer = HierarchicalWordClusterer::from_reader(clusters);

        // Then
        assert!(clusterer.is_ok());
        let clusterer = clusterer.unwrap();
        assert_eq!(
            clusterer.get_cluster("hello"),
            Some("6".to_string())
        );
        assert_eq!(
            clusterer.get_cluster("world"),
            Some("16".to_string())
        );
        assert_eq!(
            clusterer.get_cluster("\"yolo"),
            Some("24".to_string())
        );
        assert_eq!(clusterer.get_cluster("unknown"), None);
    }
}
