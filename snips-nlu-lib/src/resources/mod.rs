pub mod loading;
pub mod gazetteer;
pub mod stemmer;
pub mod word_clusterer;

use std::collections::HashMap;

use builtin_entity_parsing::CachingBuiltinEntityParser;
use resources::gazetteer::HashSetGazetteer;
use resources::stemmer::HashMapStemmer;
use resources::word_clusterer::HashMapWordClusterer;
use std::sync::Arc;

pub struct SharedResources {
    pub builtin_entity_parser: Arc<CachingBuiltinEntityParser>,
    pub gazetteers: HashMap<String, Arc<HashSetGazetteer>>,
    pub stemmer: Option<Arc<HashMapStemmer>>,
    pub word_clusterers: HashMap<String, Arc<HashMapWordClusterer>>
}
