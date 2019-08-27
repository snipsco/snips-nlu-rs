use std::ops::Range;
use std::sync::Arc;

use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind, Slot, SlotValue};

use crate::entity_parser::{BuiltinEntityParser, CustomEntity, CustomEntityParser};
use crate::errors::*;
use crate::models::nlu_engine::Entity;
use crate::utils::{EntityName, SlotName};

#[derive(Debug, Clone, PartialEq)]
pub struct InternalSlot {
    pub value: String,
    pub char_range: Range<usize>,
    pub entity: EntityName,
    pub slot_name: SlotName,
}

pub fn resolve_builtin_slot(
    internal_slot: InternalSlot,
    builtin_entities: &[BuiltinEntity],
    builtin_entity_parser: Arc<BuiltinEntityParser>,
    slots_alternatives: usize,
) -> Result<Option<Slot>> {
    let entity_kind = BuiltinEntityKind::from_identifier(&internal_slot.entity)?;
    let opt_matching_entity = match builtin_entities.iter().find(|entity| {
        entity.entity_kind == entity_kind && entity.range == internal_slot.char_range
    }) {
        Some(matching_entity) => Some(matching_entity.clone()),
        None => builtin_entity_parser
            .extract_entities(
                &internal_slot.value,
                Some(&[entity_kind]),
                false,
                slots_alternatives,
            )?
            .pop(),
    };
    Ok(opt_matching_entity.map(|entity| convert_to_builtin_slot(internal_slot, entity)))
}

pub fn resolve_custom_slot(
    internal_slot: InternalSlot,
    entity: &Entity,
    custom_entities: &[CustomEntity],
    custom_entity_parser: Arc<CustomEntityParser>,
    slots_alternatives: usize,
) -> Result<Option<Slot>> {
    let opt_matching_entity = match custom_entities.iter().find(|custom_entity| {
        custom_entity.entity_identifier == internal_slot.entity
            && custom_entity.range == internal_slot.char_range
    }) {
        Some(matching_entity) => Some(matching_entity.clone()),
        None => custom_entity_parser
            .extract_entities(
                &internal_slot.value,
                Some(&[internal_slot.entity.clone()]),
                slots_alternatives,
            )?
            .pop()
            .and_then(|entity| {
                if entity.value.chars().count() == internal_slot.value.chars().count() {
                    Some(entity)
                } else {
                    None
                }
            }),
    };
    let resolved_slot = opt_matching_entity
        .map(|matching_entity| {
            Some((
                matching_entity.resolved_value,
                matching_entity.alternative_resolved_values,
            ))
        })
        .unwrap_or_else(|| {
            if entity.automatically_extensible {
                Some((internal_slot.value.clone(), vec![]))
            } else {
                None
            }
        })
        .map(|(resolved_value, alternatives)| {
            convert_to_custom_slot(internal_slot, resolved_value, alternatives)
        });
    Ok(resolved_slot)
}

fn convert_to_custom_slot(
    slot: InternalSlot,
    resolved_value: String,
    alternatives: Vec<String>,
) -> Slot {
    let value = SlotValue::Custom(resolved_value.into());
    let alternatives = alternatives
        .into_iter()
        .map(|v| SlotValue::Custom(v.into()))
        .collect();
    Slot {
        raw_value: slot.value,
        value,
        alternatives,
        range: slot.char_range,
        entity: slot.entity,
        slot_name: slot.slot_name,
        confidence_score: None,
    }
}

fn convert_to_builtin_slot(slot: InternalSlot, builtin_entity: BuiltinEntity) -> Slot {
    Slot {
        raw_value: slot.value,
        value: builtin_entity.entity,
        alternatives: builtin_entity.alternatives,
        range: slot.char_range,
        entity: slot.entity,
        slot_name: slot.slot_name,
        confidence_score: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_parser::CachingCustomEntityParser;
    use crate::models::nlu_engine::Entity;
    use crate::testutils::*;
    use snips_nlu_ontology::*;
    use std::iter::FromIterator;
    use std::path::Path;

    #[test]
    fn test_resolve_builtin_slot() {
        // Given
        let internal_slot = InternalSlot {
            value: "8 dollars".to_string(),
            char_range: 22..31,
            slot_name: "amount".to_string(),
            entity: "snips/amountOfMoney".to_string(),
        };
        let builtin_entities = vec![
            BuiltinEntity {
                value: "5 dollars".to_string(),
                range: 5..14,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 5.0,
                    precision: Precision::Exact,
                    unit: Some("$".to_string()),
                }),
                alternatives: vec![],
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            },
            BuiltinEntity {
                value: "8 dollars".to_string(),
                range: 22..31,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 8.0,
                    precision: Precision::Exact,
                    unit: Some("$".to_string()),
                }),
                alternatives: vec![],
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            },
        ];
        let mocked_entity_parser = Arc::new(MockedBuiltinEntityParser::from_iter(vec![]));

        // When
        let resolved_slot =
            resolve_builtin_slot(internal_slot, &builtin_entities, mocked_entity_parser, 0)
                .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "8 dollars".to_string(),
            value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                value: 8.0,
                precision: Precision::Exact,
                unit: Some("$".to_string()),
            }),
            alternatives: vec![],
            range: 22..31,
            entity: "snips/amountOfMoney".to_string(),
            slot_name: "amount".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_builtin_slot_when_no_entities_found_on_whole_input() {
        // Given
        let internal_slot = InternalSlot {
            value: "5 dollars".to_string(),
            char_range: 5..14,
            slot_name: "amount".to_string(),
            entity: "snips/amountOfMoney".to_string(),
        };
        let builtin_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedBuiltinEntityParser::from_iter(vec![(
            "5 dollars".to_string(),
            vec![BuiltinEntity {
                value: "5 dollars".to_string(),
                range: 0..9,
                entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 5.0,
                    precision: Precision::Exact,
                    unit: Some("$".to_string()),
                }),
                alternatives: vec![],
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }],
        )]));

        // When
        let resolved_slot =
            resolve_builtin_slot(internal_slot, &builtin_entities, mocked_entity_parser, 0)
                .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "5 dollars".to_string(),
            value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                value: 5.0,
                precision: Precision::Exact,
                unit: Some("$".to_string()),
            }),
            alternatives: vec![],
            range: 5..14,
            entity: "snips/amountOfMoney".to_string(),
            slot_name: "amount".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_builtin_slot_with_alternatives() {
        // Given
        let internal_slot = InternalSlot {
            value: "the stones".to_string(),
            char_range: 20..30,
            slot_name: "artist".to_string(),
            entity: "snips/musicArtist".to_string(),
        };
        let builtin_entities = vec![BuiltinEntity {
            value: "the stones".to_string(),
            range: 20..30,
            entity: SlotValue::MusicArtist("The Rolling Stones".into()),
            alternatives: vec![SlotValue::MusicArtist("The Crying Stones".into())],
            entity_kind: BuiltinEntityKind::MusicArtist,
        }];
        let mocked_entity_parser = Arc::new(MockedBuiltinEntityParser::from_iter(vec![]));

        // When
        let resolved_slot =
            resolve_builtin_slot(internal_slot, &builtin_entities, mocked_entity_parser, 5)
                .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "the stones".to_string(),
            value: SlotValue::MusicArtist("The Rolling Stones".into()),
            alternatives: vec![SlotValue::MusicArtist("The Crying Stones".into())],
            range: 20..30,
            entity: "snips/musicArtist".to_string(),
            slot_name: "artist".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_custom_slot() {
        // Given
        let entity = Entity {
            automatically_extensible: false,
        };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![
            CustomEntity {
                value: "publisher".to_string(),
                range: 7..16,
                resolved_value: "Publisher".to_string(),
                alternative_resolved_values: vec![],
                entity_identifier: "userType".to_string(),
            },
            CustomEntity {
                value: "subscriber".to_string(),
                range: 27..37,
                resolved_value: "Subscriber".to_string(),
                alternative_resolved_values: vec![],
                entity_identifier: "userType".to_string(),
            },
        ];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            mocked_entity_parser,
            0,
        )
        .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("Subscriber".into()),
            alternatives: vec![],
            range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_custom_slot_when_no_entities_found_on_whole_input() {
        // Given
        let entity = Entity {
            automatically_extensible: false,
        };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![(
            "subscriber".to_string(),
            vec![CustomEntity {
                value: "subscriber".to_string(),
                range: 0..10,
                resolved_value: "Subscriber".to_string(),
                alternative_resolved_values: vec![],
                entity_identifier: "userType".to_string(),
            }],
        )]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            mocked_entity_parser,
            0,
        )
        .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("Subscriber".into()),
            alternatives: vec![],
            range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_custom_slot_when_automatically_extensible() {
        // Given
        let entity = Entity {
            automatically_extensible: true,
        };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            mocked_entity_parser,
            0,
        )
        .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("subscriber".into()),
            alternatives: vec![],
            range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_do_not_resolve_custom_slot_when_not_automatically_extensible() {
        // Given
        let entity = Entity {
            automatically_extensible: false,
        };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            mocked_entity_parser,
            0,
        )
        .unwrap();

        // Then
        let expected_result = None;
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_custom_slot_with_alternatives() {
        // Given
        let entity = Entity {
            automatically_extensible: false,
        };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![
            CustomEntity {
                value: "publisher".to_string(),
                range: 7..16,
                resolved_value: "Publisher".to_string(),
                alternative_resolved_values: vec![],
                entity_identifier: "userType".to_string(),
            },
            CustomEntity {
                value: "subscriber".to_string(),
                range: 27..37,
                resolved_value: "Subscriber".to_string(),
                alternative_resolved_values: vec!["Alternative Subscriber".to_string()],
                entity_identifier: "userType".to_string(),
            },
        ];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            mocked_entity_parser,
            5,
        )
        .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("Subscriber".into()),
            alternatives: vec![SlotValue::Custom("Alternative Subscriber".into())],
            range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn test_resolve_custom_slot_with_alternatives_when_no_entities_found_on_whole_input() {
        // Given
        let entity = Entity {
            automatically_extensible: false,
        };
        let internal_slot = InternalSlot {
            value: "invader".to_string(),
            char_range: 10..17,
            entity: "game".to_string(),
            slot_name: "game".to_string(),
        };
        let custom_entities = vec![];
        let parser_path = Path::new("data")
            .join("tests")
            .join("models")
            .join("nlu_engine_game")
            .join("custom_entity_parser");

        let custom_entity_parser = CachingCustomEntityParser::from_path(parser_path, 1000).unwrap();

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot,
            &entity,
            &custom_entities,
            Arc::new(custom_entity_parser),
            2,
        )
        .unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "invader".to_string(),
            value: SlotValue::Custom("Invader Attack 3".into()),
            alternatives: vec![
                SlotValue::Custom("Invader War Demo".into()),
                SlotValue::Custom("Space Invader Limited Edition".into()),
            ],
            range: 10..17,
            entity: "game".to_string(),
            slot_name: "game".to_string(),
            confidence_score: None,
        });
        assert_eq!(expected_result, resolved_slot);
    }
}
