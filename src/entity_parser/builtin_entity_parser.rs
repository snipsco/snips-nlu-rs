use std::path::Path;
use std::sync::Mutex;

use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind};
use snips_nlu_parsers::BuiltinEntityParser as _BuiltinEntityParser;

use super::utils::Cache;
use crate::errors::*;

pub trait BuiltinEntityParser: Send + Sync {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[BuiltinEntityKind]>,
        use_cache: bool,
    ) -> Result<Vec<BuiltinEntity>>;
}

pub struct CachingBuiltinEntityParser {
    parser: _BuiltinEntityParser,
    cache: Mutex<Cache<CacheKey, Vec<BuiltinEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<BuiltinEntityKind>,
}

impl BuiltinEntityParser for CachingBuiltinEntityParser {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[BuiltinEntityKind]>,
        use_cache: bool,
    ) -> Result<Vec<BuiltinEntity>> {
        let lowercased_sentence = sentence.to_lowercase();
        if !use_cache {
            return self
                .parser
                .extract_entities(&lowercased_sentence, filter_entity_kinds);
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
            .try_cache(&cache_key, |cache_key| {
                self.parser
                    .extract_entities(&cache_key.input, filter_entity_kinds)
            })
    }
}

impl CachingBuiltinEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        let parser = _BuiltinEntityParser::from_path(path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        Ok(Self { parser, cache })
    }
}
