use std::sync::Arc;
use ndarray::prelude::*;
use resources::SharedResources;
use snips_nlu_ontology::Language;
use builtin_entity_parsing::CachingBuiltinEntityParser;

pub fn assert_epsilon_eq_array1(a: &Array1<f32>, b: &Array1<f32>, epsilon: f32) {
    assert_eq!(a.dim(), b.dim());
    for (index, elem_a) in a.indexed_iter() {
        assert!(epsilon_eq(*elem_a, b[index], epsilon))
    }
}

pub fn epsilon_eq(a: f32, b: f32, epsilon: f32) -> bool {
    let diff = a - b;
    diff < epsilon && diff > -epsilon
}

pub fn english_shared_resources() -> Arc<SharedResources> {
    let builtin_entity_parser = CachingBuiltinEntityParser::from_language(Language::EN, 1000).unwrap();
    Arc::new(SharedResources { builtin_entity_parser })
}
