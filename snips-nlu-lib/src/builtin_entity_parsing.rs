use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lru_cache::LruCache;

use snips_nlu_ontology::{BuiltinEntityKind, BuiltinEntity, Language};
use snips_nlu_ontology_parsers::BuiltinEntityParser;


pub struct CachingBuiltinEntityParser {
    parser: BuiltinEntityParser,
    cache: Mutex<EntityCache>,
}

impl CachingBuiltinEntityParser {
    pub fn new(lang: Language, cache_capacity: usize) -> Self {
        CachingBuiltinEntityParser {
            parser: BuiltinEntityParser::new(lang),
            cache: Mutex::new(EntityCache::new(cache_capacity)),
        }
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
        let cached_value = self.0.get_mut(key).map(|a| a.clone());
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

lazy_static! {
    static ref CACHED_PARSERS: Mutex<HashMap<Language, Arc<CachingBuiltinEntityParser>>> =
        Mutex::new(HashMap::new());
}

pub struct BuiltinEntityParserFactory;

impl BuiltinEntityParserFactory {
    pub fn get(lang: Language) -> Arc<CachingBuiltinEntityParser> {
        CACHED_PARSERS
            .lock()
            .unwrap()
            .entry(lang)
            .or_insert_with(|| Arc::new(CachingBuiltinEntityParser::new(lang, 1000)))
            .clone()
    }

    pub fn clean() {
        CACHED_PARSERS
            .lock()
            .unwrap()
            .clear();
    }
}
