use std::str::FromStr;

use builtin_entities::{BuiltinEntityKind, RustlingParser};
use itertools::Itertools;
use pipeline::nlu_engine::TaggedEntity;
use rustling_ontology::Lang;
use utils::miscellaneous::ranges_overlap;

const TAGGING_SCOPE: [BuiltinEntityKind; 2] = [BuiltinEntityKind::Duration, BuiltinEntityKind::Time];

pub fn enrich_entities(mut tagged_entities: Vec<TaggedEntity>,
                       other_tagged_entities: Vec<TaggedEntity>) -> Vec<TaggedEntity> {
    for entity in other_tagged_entities {
        if tagged_entities.iter().find(|e| ranges_overlap(&e.range, &entity.range)).is_none() {
            tagged_entities.push(entity);
        }
    }
    tagged_entities
}

pub fn tag_builtin_entities(text: &str, language: &str) -> Vec<TaggedEntity> {
    Lang::from_str(language)
        .ok()
        .map(|rustling_lang|
            RustlingParser::get(rustling_lang)
                .extract_entities(text, Some(&TAGGING_SCOPE))
                .into_iter()
                .map(|entity|
                    TaggedEntity {
                        value: entity.value,
                        range: entity.range,
                        entity: entity.entity_kind.identifier().to_string(),
                        slot_name: None
                    })
                .collect_vec())
        .unwrap_or(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrich_entities_works() {
        // Given
        let tagged_entities = vec![
            TaggedEntity { value: "hello world".to_string(), range: 0..11, entity: "entity1".to_string(), slot_name: None },
            TaggedEntity { value: "!!!".to_string(), range: 13..16, entity: "entity2".to_string(), slot_name: None },
        ];

        let other_tagged_entities = vec![
            TaggedEntity { value: "world".to_string(), range: 6..11, entity: "entity1".to_string(), slot_name: None },
            TaggedEntity { value: "yay".to_string(), range: 16..19, entity: "entity3".to_string(), slot_name: None },
        ];

        // When
        let enriched_entities = enrich_entities(tagged_entities, other_tagged_entities);

        // Then
        let expected_entities = vec![
            TaggedEntity { value: "hello world".to_string(), range: 0..11, entity: "entity1".to_string(), slot_name: None },
            TaggedEntity { value: "!!!".to_string(), range: 13..16, entity: "entity2".to_string(), slot_name: None },
            TaggedEntity { value: "yay".to_string(), range: 16..19, entity: "entity3".to_string(), slot_name: None },
        ];

        assert_eq!(expected_entities, enriched_entities);
    }
}
