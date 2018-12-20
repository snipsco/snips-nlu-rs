use std::hash::Hash;

use lru_cache::LruCache;

use crate::errors::*;

pub struct Cache<K, V>(LruCache<K, V>)
where
    K: Eq + Hash + Clone,
    V: Clone;

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Cache(LruCache::new(capacity))
    }

    pub fn try_cache<F: Fn(&K) -> Result<V>>(&mut self, key: &K, producer: F) -> Result<V> {
        let cached_value = self.0.get_mut(key).cloned();
        if let Some(value) = cached_value {
            return Ok(value);
        }
        let value = producer(key)?;
        self.0.insert(key.clone(), value.clone());
        Ok(value)
    }
}
