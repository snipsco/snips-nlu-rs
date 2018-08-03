pub mod loading;
pub mod gazetteer;
pub mod stemmer;
pub mod word_clusterer;

use builtin_entity_parsing::CachingBuiltinEntityParser;

pub struct SharedResources {
    pub builtin_entity_parser: CachingBuiltinEntityParser
}
