pub mod loading;
pub mod gazetteer;
pub mod stemmer;
pub mod word_clusterer;

use std::collections::HashMap;
use std::sync::Arc;

use entity_parser::{CachingBuiltinEntityParser, CachingCustomEntityParser};
use resources::gazetteer::HashSetGazetteer;
use resources::stemmer::HashMapStemmer;
use resources::word_clusterer::HashMapWordClusterer;

pub struct SharedResources {
    pub builtin_entity_parser: Arc<CachingBuiltinEntityParser>,
    pub custom_entity_parser: Arc<CachingCustomEntityParser>,
    pub gazetteers: HashMap<String, Arc<HashSetGazetteer>>,
    pub stemmer: Option<Arc<HashMapStemmer>>,
    pub word_clusterers: HashMap<String, Arc<HashMapWordClusterer>>
}
