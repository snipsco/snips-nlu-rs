use std::path::Path;
use std::sync::Mutex;

use entity_parser::utils::Cache;
use errors::*;
use snips_nlu_ontology::{BuiltinEntityKind, BuiltinEntity};
use snips_nlu_ontology_parsers::BuiltinEntityParser;
#[cfg(test)]
use snips_nlu_ontology::Language;
#[cfg(test)]
use snips_nlu_ontology_parsers::BuiltinEntityParserLoader;

pub struct CachingBuiltinEntityParser {
    parser: BuiltinEntityParser,
    cache: Mutex<Cache<CacheKey, Vec<BuiltinEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<BuiltinEntityKind>,
}

impl CachingBuiltinEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        let parser = BuiltinEntityParser::from_path(path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        Ok(Self { parser, cache })
    }

    pub fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[BuiltinEntityKind]>,
        use_cache: bool,
    ) -> Vec<BuiltinEntity> {
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
            .cache(&cache_key,
                   |cache_key| self.parser.extract_entities(&cache_key.input, filter_entity_kinds))
            .clone()
    }
}

#[cfg(test)]
impl CachingBuiltinEntityParser {
    pub fn from_language(language: Language, cache_capacity: usize) -> Result<Self> {
        let parser = BuiltinEntityParserLoader::new(language).load()?;
        let cache = Mutex::new(EntityCache::new(cache_capacity));
        Ok(Self { parser, cache })
    }
}

