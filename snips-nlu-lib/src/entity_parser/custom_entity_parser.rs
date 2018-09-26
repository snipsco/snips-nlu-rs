use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use entity_parser::utils::Cache;
use errors::*;
use failure::ResultExt;
use serde_json;
use snips_nlu_ontology::Language;
use snips_nlu_ontology_parsers::{GazetteerParser, GazetteerEntityMatch};
use utils::EntityName;

pub type CustomEntity = GazetteerEntityMatch<String>;

pub trait CustomEntityParser: Send + Sync {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[String]>,
        use_cache: bool,
    ) -> Result<Vec<CustomEntity>>;
}

pub struct CachingCustomEntityParser {
    language: Language,
    parser: GazetteerParser<String>,
    cache: Mutex<Cache<CacheKey, Vec<CustomEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<EntityName>,
}

impl CustomEntityParser for CachingCustomEntityParser {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[String]>,
        use_cache: bool,
    ) -> Result<Vec<CustomEntity>> {
        let lowercased_sentence = sentence.to_lowercase();
        if !use_cache {
            return self.parser.extract_entities(&lowercased_sentence, filter_entity_kinds);
        }
        let cache_key = CacheKey {
            input: lowercased_sentence,
            kinds: filter_entity_kinds
                .map(|entity_kinds| entity_kinds.to_vec())
                .unwrap_or_else(|| vec![]),
        };

        self.cache
            .lock()
            .unwrap()
            .try_cache(&cache_key, |cache_key| self.parser.extract_entities(&cache_key.input, filter_entity_kinds))
    }
}

#[derive(Deserialize)]
struct CustomEntityParserMetadata {
    language: String,
    parser_directory: String,
}

impl CachingCustomEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        let metadata_path = path.as_ref().join("metadata.json");
        let metadata_file = File::open(&metadata_path)
            .with_context(|_|
                format!("Cannot open metadata file for custom entity parser at path: {:?}",
                        metadata_path))?;
        let metadata: CustomEntityParserMetadata = serde_json::from_reader(metadata_file)
            .with_context(|_| "Cannot deserialize custom entity parser metadata")?;
        let language = Language::from_str(&metadata.language)?;
        let gazetteer_parser_path = path.as_ref().join(&metadata.parser_directory);
        let parser = GazetteerParser::from_path(gazetteer_parser_path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        Ok(Self { language, parser, cache })
    }
}
