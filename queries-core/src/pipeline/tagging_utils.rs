use std::str::FromStr;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use builtin_entities::{BuiltinEntityKind, RustlingParser};
use itertools::Itertools;
use pipeline::nlu_engine::PartialTaggedEntity;
use rustling_ontology::Lang;
use nlu_utils::range::ranges_overlap;

const TAGGING_SCOPE: [BuiltinEntityKind; 2] = [BuiltinEntityKind::Duration, BuiltinEntityKind::Time];

pub fn enrich_entities(mut tagged_entities: Vec<PartialTaggedEntity>,
                       other_tagged_entities: Vec<PartialTaggedEntity>) -> Vec<PartialTaggedEntity> {
    for entity in other_tagged_entities {
        let is_overlapping = tagged_entities.iter()
            .find(|e| {
                if let (Some(r1), Some(r2)) = (e.range.as_ref(), entity.range.as_ref()) {
                    ranges_overlap(&r1, &r2)
                } else {
                    false
                }
            })
            .is_some();
        if !is_overlapping {
            tagged_entities.push(entity);
        }
    }
    tagged_entities
}

pub fn tag_builtin_entities(text: &str, language: &str) -> Vec<PartialTaggedEntity> {
    let tagging_scope = TAGGING_SCOPE;
    let tagging_scope_set: HashSet<&BuiltinEntityKind> = HashSet::from_iter(tagging_scope.iter());
    Lang::from_str(language)
        .ok()
        .map(|rustling_lang|
            RustlingParser::get(rustling_lang)
                .extract_entities(text, None)
                .into_iter()
                .filter_map(|entity| {
                    if tagging_scope_set.contains(&entity.entity_kind) {
                        Some(PartialTaggedEntity {
                            value: entity.value,
                            range: Some(entity.range),
                            entity: entity.entity_kind.identifier().to_string(),
                            slot_name: None
                        })
                    } else {
                        None
                    }
                })
                .collect_vec())
        .unwrap_or(vec![])
}

impl PartialTaggedEntity {
    fn update_with_slot_name(&self, slot_name: String) -> PartialTaggedEntity {
        PartialTaggedEntity {
            value: self.value.clone(),
            range: self.range.clone(),
            entity: self.entity.clone(),
            slot_name: Some(slot_name)
        }
    }
}

pub fn disambiguate_tagged_entities(tagged_entities: Vec<PartialTaggedEntity>,
                                    slot_name_mapping: HashMap<String, String>) -> Vec<PartialTaggedEntity> {
    let mut entity_to_slots_mapping = HashMap::<String, Vec<String>>::new();
    for (slot_name, entity) in slot_name_mapping.into_iter() {
        let slot_names = entity_to_slots_mapping.entry(entity).or_insert(vec![]);
        slot_names.push(slot_name)
    }
    tagged_entities
        .into_iter()
        .map(|tagged_entity|
            if tagged_entity.slot_name.is_some() {
                tagged_entity
            } else {
                if let Some(slot_names) = entity_to_slots_mapping.get(&tagged_entity.entity) {
                    // Check slot_name ambiguity
                    if slot_names.len() == 1 {
                        tagged_entity.update_with_slot_name(slot_names[0].clone())
                    } else {
                        tagged_entity
                    }
                } else {
                    tagged_entity
                }
            }
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrich_entities_works() {
        // Given
        let tagged_entities = vec![
            PartialTaggedEntity { value: "hello world".to_string(), range: Some(0..11), entity: "entity1".to_string(), slot_name: None },
            PartialTaggedEntity { value: "!!!".to_string(), range: Some(13..16), entity: "entity2".to_string(), slot_name: None },
        ];

        let other_tagged_entities = vec![
            PartialTaggedEntity { value: "world".to_string(), range: Some(6..11), entity: "entity1".to_string(), slot_name: None },
            PartialTaggedEntity { value: "yay".to_string(), range: Some(16..19), entity: "entity3".to_string(), slot_name: None },
        ];

        // When
        let enriched_entities = enrich_entities(tagged_entities, other_tagged_entities);

        // Then
        let expected_entities = vec![
            PartialTaggedEntity { value: "hello world".to_string(), range: Some(0..11), entity: "entity1".to_string(), slot_name: None },
            PartialTaggedEntity { value: "!!!".to_string(), range: Some(13..16), entity: "entity2".to_string(), slot_name: None },
            PartialTaggedEntity { value: "yay".to_string(), range: Some(16..19), entity: "entity3".to_string(), slot_name: None },
        ];

        assert_eq!(expected_entities, enriched_entities);
    }

    #[test]
    fn disambiguate_tagged_entities_works() {
        // Given
        let tagged_entities = vec![
            PartialTaggedEntity {
                value: "abc".to_string(),
                range: Some(0..3),
                entity: "entity_4".to_string(),
                slot_name: Some("slot_5".to_string())
            },
            PartialTaggedEntity {
                value: "def".to_string(),
                range: Some(13..16),
                entity: "entity_1".to_string(),
                slot_name: None
            },
            PartialTaggedEntity {
                value: "ghi".to_string(),
                range: Some(20..23),
                entity: "entity_2".to_string(),
                slot_name: Some("slot_3".to_string())
            },
            PartialTaggedEntity {
                value: "ghi".to_string(),
                range: Some(26..29),
                entity: "entity_2".to_string(),
                slot_name: None
            },
        ];

        let slot_name_mapping = hashmap! {
            "slot_1".to_string() => "entity_1".to_string(),
            "slot_2".to_string() => "entity_1".to_string(),
            "slot_3".to_string() => "entity_2".to_string(),
            "slot_4".to_string() => "entity_3".to_string(),
            "slot_5".to_string() => "entity_4".to_string(),
        };

        // When
        let result = disambiguate_tagged_entities(tagged_entities, slot_name_mapping);

        // Then
        let expected_result = vec![
            PartialTaggedEntity {
                value: "abc".to_string(),
                range: Some(0..3),
                entity: "entity_4".to_string(),
                slot_name: Some("slot_5".to_string())
            },
            PartialTaggedEntity {
                value: "def".to_string(),
                range: Some(13..16),
                entity: "entity_1".to_string(),
                slot_name: None
            },
            PartialTaggedEntity {
                value: "ghi".to_string(),
                range: Some(20..23),
                entity: "entity_2".to_string(),
                slot_name: Some("slot_3".to_string())
            },
            PartialTaggedEntity {
                value: "ghi".to_string(),
                range: Some(26..29),
                entity: "entity_2".to_string(),
                slot_name: Some("slot_3".to_string())
            },
        ];

        assert_eq!(expected_result, result);
    }
}
