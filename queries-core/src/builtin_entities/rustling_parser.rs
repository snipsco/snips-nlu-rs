use std::sync::{Mutex, Arc};
use std::ops::Range;
use std::collections::HashMap;
use std::time::Instant;

use itertools::Itertools;

use core_ontology::SlotValue;
use rustling_ontology::{Lang, Parser, build_parser, ResolverContext};
use builtin_entities::ontology::*;

pub struct RustlingParser {
    parser: Parser,
    lang: Lang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustlingEntity {
    pub value: String,
    pub range: Range<usize>,
    pub entity: SlotValue,
    pub entity_kind: BuiltinEntityKind,
}

impl RustlingParser {
    pub fn get(lang: Lang) -> Arc<RustlingParser> {
        lazy_static! {
            static ref CACHED_PARSERS: Mutex<HashMap<String, Arc<RustlingParser>>> = Mutex::new(HashMap::new());
        }

        CACHED_PARSERS.lock().unwrap()
            .entry(lang.to_string())
            .or_insert_with(|| Arc::new(RustlingParser { parser: build_parser(lang).unwrap(), lang }))
            .clone()
    }

    pub fn extract_entities(&self,
                            sentence: &str,
                            filter_entity_kinds: Option<&[BuiltinEntityKind]>) -> Vec<RustlingEntity> {
        lazy_static! {
            static ref CACHED_ENTITY: Mutex<EntityCache> = Mutex::new(EntityCache::new(60));
        }

        let key = CacheKey {
            lang: self.lang.to_string(),
            input: sentence.into(),
            kinds: filter_entity_kinds.map(|kinds| kinds.to_vec())
        };
        CACHED_ENTITY.lock().unwrap().cache(&key, |key| {
            let context = ResolverContext::default();
            if let Some(kinds) = key.kinds.as_ref() {
                let kind_order = kinds.iter().map(|kind| kind.dimension_kind()).collect::<Vec<_>>();
                self.parser
                    .parse_with_kind_order(&sentence.to_lowercase(), &context, &kind_order)
                    .unwrap_or(Vec::new())
                    .iter()
                    .filter_map(|m| {
                        let entity_kind = BuiltinEntityKind::from_rustling_output(&m.value);
                        kinds.iter()
                            .find(|kind| **kind == entity_kind)
                            .map(|kind|
                                RustlingEntity {
                                    value: sentence[m.byte_range.0..m.byte_range.1].into(),
                                    range: m.char_range.0..m.char_range.1,
                                    entity: SlotValue::from_rustling(m.value.clone()),
                                    entity_kind: kind.clone()
                                })
                    })
                    .sorted_by(|a, b| Ord::cmp(&a.range.start, &b.range.start))
            } else {
                self.parser.parse(&sentence.to_lowercase(), &context)
                    .unwrap_or(Vec::new())
                    .iter()
                    .map(|entity|
                        RustlingEntity {
                            value: sentence[entity.byte_range.0..entity.byte_range.1].into(),
                            range: entity.char_range.0..entity.char_range.1,
                            entity: SlotValue::from_rustling(entity.value.clone()),
                            entity_kind: BuiltinEntityKind::from_rustling_output(&entity.value)
                        })
                    .sorted_by(|a, b| Ord::cmp(&a.range.start, &b.range.start))
            }
        }).entities
    }
}

struct EntityCache {
    container: HashMap<CacheKey, CacheValue>,
    valid_duration_sec: u64,
}

impl EntityCache {
    fn new(valid_duration_sec: u64) -> EntityCache {
        EntityCache { container: HashMap::new(), valid_duration_sec }
    }

    fn cache<F: Fn(&CacheKey) -> Vec<RustlingEntity>>(&mut self, key: &CacheKey, producer: F) -> CacheValue {
        let cached_value = self.container.get(key).map(|a| a.clone());
        if let Some(value) = cached_value { if value.is_valid(self.valid_duration_sec) { return value; } }
        let value = CacheValue::new(producer(key));
        self.container.insert(key.clone(), value.clone());
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    lang: String,
    input: String,
    kinds: Option<Vec<BuiltinEntityKind>>,
}

#[derive(Debug, Clone)]
struct CacheValue {
    entities: Vec<RustlingEntity>,
    instant: Instant,
}

impl CacheValue {
    fn new(entities: Vec<RustlingEntity>) -> CacheValue {
        CacheValue {
            entities: entities,
            instant: Instant::now(),
        }
    }

    fn is_valid(&self, valid_duration_sec: u64) -> bool {
        self.instant.elapsed().as_secs() < valid_duration_sec
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core_ontology::*;
    use itertools::Itertools;

    #[test]
    fn test_entities_extraction() {
        let parser = RustlingParser::get(Lang::EN);
        assert_eq!(vec![
            BuiltinEntityKind::Number,
            BuiltinEntityKind::Time,
        ],
        parser.extract_entities("Book me restaurant for two people tomorrow", None)
            .iter()
            .map(|e| e.entity_kind)
            .collect_vec());

        assert_eq!(vec![
            BuiltinEntityKind::Duration,
        ],
        parser.extract_entities("The weather during two weeks", None)
            .iter()
            .map(|e| e.entity_kind)
            .collect_vec());
    }

    #[test]
    fn test_entity_cache() {
        fn parse(_: &CacheKey) -> Vec<RustlingEntity> {
            vec![
                RustlingEntity {
                    value: "two".into(),
                    range: 23..26,
                    entity_kind: BuiltinEntityKind::Number,
                    entity: SlotValue::Number(NumberValue(2.0))
                },
                RustlingEntity {
                    value: "4.5".into(),
                    range: 34..42,
                    entity_kind: BuiltinEntityKind::Number,
                    entity: SlotValue::Number(NumberValue(4.5))
                },
            ]
        }

        let key = CacheKey { lang: "en".into(), input: "test".into(), kinds: None };

        let mut cache = EntityCache::new(10); // caching for 10s
        assert_eq!(cache.cache(&key, parse).instant, cache.cache(&key, parse).instant);

        let mut cache = EntityCache::new(0); // no caching
        assert_ne!(cache.cache(&key, parse).instant, cache.cache(&key, parse).instant);
    }
}
