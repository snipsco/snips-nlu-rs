use std::sync::{Mutex, Arc};
use std::ops::Range;
use std::collections::HashMap;
use std::time::Instant;
use errors::*;
use rustling_ontology::{Lang, Parser, DimensionKind, build_parser, ParsingContext, Output};

pub struct RustlingParser {
    parser: Parser,
    lang: Lang,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RustlingEntity {
    pub value: String,
    pub range: Range<usize>,
    pub char_range: Range<usize>,
    pub kind: EntityKind,
}

#[derive(Copy, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Time,
    Duration,
    Number,
}

impl EntityKind {
    fn all() -> Vec<EntityKind> {
        vec![EntityKind::Time, EntityKind::Duration, EntityKind::Number]
    }
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
                            filter_entity_kinds: Option<Vec<EntityKind>>) -> Vec<RustlingEntity> {
        lazy_static! {
            static ref CACHED_ENTITY: Mutex<EntityCache> = Mutex::new(EntityCache::new(60));
        }

        let key = CacheKey { lang: "en".into(), input: "test".into(), kinds: filter_entity_kinds.unwrap_or_else(|| EntityKind::all())};
        CACHED_ENTITY.lock().unwrap().cache(&key, |key| {
            let context = ParsingContext::default();
            let kind_order = key.kinds.iter().map(|kind| kind.dimension_kind()).collect::<Vec<_>>();
            let mut entities = self.parser.parse_with_kind_order(&sentence.to_lowercase(), &context, &kind_order, true)
                .unwrap_or(Vec::new())
               .iter()
               .filter_map(|m| {
                   EntityKind::from_rustling_output(&m.value)
                       .map(|kind| {
                           RustlingEntity {
                               value: sentence[m.byte_range.0..m.byte_range.1].into(),
                               range: m.byte_range.0..m.byte_range.1,
                               char_range: m.char_range.0..m.char_range.1,
                               kind,
                           }
                       })
               })
               .collect::<Vec<_>>();
               entities.sort_by_key(|e| e.range.start);
               entities

        }).entities
    }
}

impl EntityKind {
    pub fn identifier(&self) -> &str {
        match *self {
            EntityKind::Time => "snips/datetime",
            EntityKind::Number => "snips/number",
            EntityKind::Duration => "snips/duration",
        }
    }

    pub fn from_identifier(identifier: &str) -> Result<EntityKind> {
        Self::all()
            .into_iter()
            .find(|kind| kind.identifier() == identifier)
            .ok_or(format!("Unknown EntityKind identifier: {}", identifier).into())
    }

    fn from_rustling_output(v: &Output) -> Option<EntityKind> {
        match *v {
            Output::Time(_) => Some(EntityKind::Time),
            Output::TimeInterval(_) => Some(EntityKind::Time),
            Output::Integer(_) => Some(EntityKind::Number),
            Output::Float(_) => Some(EntityKind::Number),
            Output::Duration(_) => Some(EntityKind::Duration),
            _ => None
        }
    }

    fn dimension_kind(&self) -> DimensionKind {
        match *self {
            EntityKind::Time => DimensionKind::Time,
            EntityKind::Number => DimensionKind::Number,
            EntityKind::Duration => DimensionKind::Duration,
        }
    }
}

struct EntityCache {
    container: HashMap<CacheKey, CacheValue>,
    valid_duration_sec: u64,
}

impl EntityCache {
    fn new(valid_duration_sec: u64) -> EntityCache {
        EntityCache { container: HashMap::new(), valid_duration_sec, }
    }

    fn cache<F: Fn(&CacheKey) -> Vec<RustlingEntity>>(&mut self, key: &CacheKey, producer: F) -> CacheValue {
        let cached_value = self.container.get(key).map(|a| a.clone());
        if let Some(value) = cached_value { if value.is_valid(self.valid_duration_sec) { return value } }
        let value  = CacheValue::new(producer(key));
        self.container.insert(key.clone(), value.clone());
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    lang: String,
    input: String,
    kinds: Vec<EntityKind>,
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

    #[test]
    fn test_entities_extraction() {
        let parser = RustlingParser::get(Lang::EN);
        assert_eq!(vec![
            RustlingEntity { value: "two".into(), range: 23..26, char_range: 23..26, kind: EntityKind::Number },
            RustlingEntity { value: "tomorrow".into(), range: 34..42, char_range: 34..42, kind: EntityKind::Time },
        ], parser.extract_entities("Book me restaurant for two people tomorrow", None));

        assert_eq!(vec![
            RustlingEntity { value: "two weeks".into(), range: 19..28, char_range: 19..28, kind: EntityKind::Duration },
        ], parser.extract_entities("The weather during two weeks", None));
    }

    #[test]
    fn test_entity_cache() {
        fn parse(key: &CacheKey) -> Vec<RustlingEntity> {
            vec![
                RustlingEntity { value: "two".into(), range: 23..26, char_range: 23..26, kind: EntityKind::Number },
                RustlingEntity { value: "tomorrow".into(), range: 34..42, char_range: 34..42, kind: EntityKind::Time },
            ]
        }

        let key = CacheKey { lang: "en".into(), input: "test".into(), kinds: vec![] };
        
        let mut cache = EntityCache::new(10); // caching for 10s
        assert_eq!(cache.cache(&key, parse).instant, cache.cache(&key, parse).instant);

        let mut cache = EntityCache::new(0); // no caching
        assert_ne!(cache.cache(&key, parse).instant, cache.cache(&key, parse).instant);
    }
}
