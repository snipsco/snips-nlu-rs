use std::path::Path;
use std::sync::Mutex;

use entity_parser::utils::Cache;
use errors::*;
use snips_nlu_ontology_parsers::{GazetteerParser, GazetteerEntityMatch};
use utils::EntityName;

pub type CustomEntity = GazetteerEntityMatch<String>;

pub struct CachingCustomEntityParser {
    parser: GazetteerParser<String>,
    cache: Mutex<Cache<CacheKey, Vec<CustomEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<EntityName>,
}

impl CachingCustomEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        let parser = GazetteerParser::from_path(path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        Ok(Self { parser, cache })
    }

    pub fn extract_entities<'a>(
        &'a self,
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
