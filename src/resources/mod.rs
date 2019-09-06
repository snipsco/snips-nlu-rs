pub mod gazetteer;
pub mod loading;
pub mod stemmer;
pub mod word_clusterer;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use self::gazetteer::Gazetteer;
use self::stemmer::Stemmer;
use self::word_clusterer::WordClusterer;
use super::entity_parser::{BuiltinEntityParser, CustomEntityParser};

pub struct SharedResources {
    pub builtin_entity_parser: Arc<dyn BuiltinEntityParser>,
    pub custom_entity_parser: Arc<dyn CustomEntityParser>,
    pub gazetteers: HashMap<String, Arc<dyn Gazetteer>>,
    pub stemmer: Option<Arc<dyn Stemmer>>,
    pub word_clusterers: HashMap<String, Arc<dyn WordClusterer>>,
    pub stop_words: HashSet<String>,
}
