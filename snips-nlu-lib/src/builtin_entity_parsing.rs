use std::sync::Mutex;

use lru_cache::LruCache;

use errors::*;
use failure::ResultExt;
use snips_nlu_ontology::{BuiltinEntityKind, BuiltinEntity};
#[cfg(test)]
use snips_nlu_ontology::Language;
use snips_nlu_ontology_parsers::{BuiltinEntityParser, BuiltinEntityParserConfiguration};

pub struct CachingBuiltinEntityParser {
    parser: BuiltinEntityParser,
    cache: Mutex<EntityCache>,
}

// TODO: fix this
unsafe impl Send for CachingBuiltinEntityParser {}

impl CachingBuiltinEntityParser {
    pub fn new(
        configuration: BuiltinEntityParserConfiguration,
        cache_capacity: usize,
    ) -> Result<Self> {
        let parser = BuiltinEntityParser::new(configuration)
            .with_context(|_| BuiltinEntityParserError::LoadingError)?;
        Ok(Self {
            parser,
            cache: Mutex::new(EntityCache::new(cache_capacity)),
        })
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
    }
}

#[cfg(test)]
impl CachingBuiltinEntityParser {
    pub fn from_language(language: Language, cache_capacity: usize) -> Result<Self> {
        let configuration = BuiltinEntityParserConfiguration {
            language,
            gazetteer_entity_configurations: vec![]
        };
        CachingBuiltinEntityParser::new(configuration, cache_capacity)
    }
}

struct EntityCache(LruCache<CacheKey, Vec<BuiltinEntity>>);

impl EntityCache {
    fn new(capacity: usize) -> Self {
        EntityCache(LruCache::new(capacity))
    }

    fn cache<F: Fn(&CacheKey) -> Vec<BuiltinEntity>>(
        &mut self,
        key: &CacheKey,
        producer: F,
    ) -> Vec<BuiltinEntity> {
        let cached_value = self.0.get_mut(key).cloned();
        if let Some(value) = cached_value {
            return value;
        }
        let value = producer(key);
        self.0.insert(key.clone(), value.clone());
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<BuiltinEntityKind>,
}
