pub mod loading;
pub mod gazetteer;
pub mod stemmer;
pub mod word_clusterer;

use std::collections::HashMap;
use std::sync::Arc;

use entity_parser::{BuiltinEntityParser, CustomEntityParser};
use resources::gazetteer::Gazetteer;
use resources::stemmer::Stemmer;
use resources::word_clusterer::WordClusterer;

pub struct SharedResources {
    pub builtin_entity_parser: Arc<BuiltinEntityParser>,
    pub custom_entity_parser: Arc<CustomEntityParser>,
    pub gazetteers: HashMap<String, Arc<Gazetteer>>,
    pub stemmer: Option<Arc<Stemmer>>,
    pub word_clusterers: HashMap<String, Arc<WordClusterer>>,
}
