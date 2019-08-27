use std::path::Path;
use std::sync::Mutex;

use log::info;
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
        max_alternative_resolved_values: usize,
    ) -> Result<Vec<BuiltinEntity>>;
}

pub struct CachingBuiltinEntityParser {
    parser: _BuiltinEntityParser,
    cache: Mutex<Cache<CacheKey, Vec<BuiltinEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Option<Vec<BuiltinEntityKind>>,
    max_alternative_resolved_values: usize,
}

impl BuiltinEntityParser for CachingBuiltinEntityParser {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[BuiltinEntityKind]>,
        use_cache: bool,
        max_alternative_resolved_values: usize,
    ) -> Result<Vec<BuiltinEntity>> {
        let lowercased_sentence = sentence.to_lowercase();
        if !use_cache {
            return self.parser.extract_entities(
                &lowercased_sentence,
                filter_entity_kinds,
                max_alternative_resolved_values,
            );
        }
        let cache_key = CacheKey {
            input: lowercased_sentence,
            kinds: filter_entity_kinds.map(|entity_kinds| entity_kinds.to_vec()),
            max_alternative_resolved_values,
        };

        self.cache
            .lock()
            .unwrap()
            .try_cache(&cache_key, |cache_key| {
                self.parser.extract_entities(
                    &cache_key.input,
                    filter_entity_kinds,
                    max_alternative_resolved_values,
                )
            })
    }
}

impl CachingBuiltinEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        info!("Loading builtin entity parser ({:?}) ...", path.as_ref());
        let parser = _BuiltinEntityParser::from_path(path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        info!("Builtin entity parser loaded");
        Ok(Self { parser, cache })
    }
}
