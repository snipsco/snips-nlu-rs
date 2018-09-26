use std::ops::Range;
use std::sync::Arc;

use entity_parser::{CustomEntityParser, CustomEntity, BuiltinEntityParser};
use errors::*;
use models::nlu_engine::Entity;
use snips_nlu_ontology::{BuiltinEntity, BuiltinEntityKind, Slot, SlotValue};
use utils::{EntityName, SlotName};

#[derive(Debug, Clone, PartialEq)]
pub struct InternalSlot {
    pub value: String,
    pub char_range: Range<usize>,
    pub entity: EntityName,
    pub slot_name: SlotName,
}

pub fn resolve_builtin_slot(
    internal_slot: InternalSlot,
    builtin_entities: &Vec<BuiltinEntity>,
    builtin_entity_parser: Arc<BuiltinEntityParser>,
) -> Result<Option<Slot>> {
    let entity_kind = BuiltinEntityKind::from_identifier(&internal_slot.entity)?;
    let resolved_slot = builtin_entities
        .iter()
        .find(|entity|
            entity.entity_kind == entity_kind && entity.range == internal_slot.char_range
        )
        .map(|builtin_entity| Some(builtin_entity.entity.clone()))
        .unwrap_or_else(||
            builtin_entity_parser
                .extract_entities(&internal_slot.value, Some(&[entity_kind]), false)
                .pop()
                .map(|builtin_entity| builtin_entity.entity)
        )
        .map(|entity| convert_to_builtin_slot(internal_slot, entity));
    Ok(resolved_slot)
}

pub fn resolve_custom_slot(
    internal_slot: InternalSlot,
    entity: &Entity,
    custom_entities: &Vec<CustomEntity>,
    custom_entity_parser: Arc<CustomEntityParser>,
) -> Result<Option<Slot>> {
    let opt_matching_entity = match custom_entities
        .into_iter()
        .find(|custom_entity|
            custom_entity.entity_identifier == internal_slot.entity &&
                custom_entity.range == internal_slot.char_range
        ) {
        Some(matching_entity) => Some(matching_entity.clone()),
        None => custom_entity_parser
            .extract_entities(&internal_slot.value, Some(&[internal_slot.entity.clone()]), true)?
            .pop()
    };
    let resolved_slot = opt_matching_entity
        .map(|matching_entity| Some(matching_entity.resolved_value))
        .unwrap_or_else(||
            if entity.automatically_extensible {
                Some(internal_slot.value.clone())
            } else {
                None
            })
        .map(|resolved_value| convert_to_custom_slot(internal_slot, resolved_value));
    Ok(resolved_slot)
}

fn convert_to_custom_slot(
    slot: InternalSlot,
    resolved_value: String,
) -> Slot {
    let value = SlotValue::Custom(resolved_value.into());
    Slot {
        raw_value: slot.value,
        value,
        range: Some(slot.char_range),
        entity: slot.entity,
        slot_name: slot.slot_name,
    }
}

fn convert_to_builtin_slot(slot: InternalSlot, slot_value: SlotValue) -> Slot {
    Slot {
        raw_value: slot.value,
        value: slot_value,
        range: Some(slot.char_range),
        entity: slot.entity,
        slot_name: slot.slot_name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use snips_nlu_ontology::*;
    use models::nlu_engine::Entity;
    use testutils::*;

    #[test]
    fn should_resolve_builtin_slot() {
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
                entity_kind: BuiltinEntityKind::AmountOfMoney,
            }
        ];
        let mocked_entity_parser = Arc::new(MockedBuiltinEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_builtin_slot(
            internal_slot, &builtin_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "8 dollars".to_string(),
            value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                value: 8.0,
                precision: Precision::Exact,
                unit: Some("$".to_string()),
            }),
            range: Some(22..31),
            entity: "snips/amountOfMoney".to_string(),
            slot_name: "amount".to_string(),
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn should_resolve_builtin_slot_when_no_entities_found_on_whole_input() {
        // Given
        let internal_slot = InternalSlot {
            value: "5 dollars".to_string(),
            char_range: 5..14,
            slot_name: "amount".to_string(),
            entity: "snips/amountOfMoney".to_string(),
        };
        let builtin_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedBuiltinEntityParser::from_iter(
            vec![(
                "5 dollars".to_string(),
                vec![
                    BuiltinEntity {
                        value: "5 dollars".to_string(),
                        range: 0..9,
                        entity: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                            value: 5.0,
                            precision: Precision::Exact,
                            unit: Some("$".to_string()),
                        }),
                        entity_kind: BuiltinEntityKind::AmountOfMoney,
                    }]
            )]));

        // When
        let resolved_slot = resolve_builtin_slot(
            internal_slot, &builtin_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "5 dollars".to_string(),
            value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                value: 5.0,
                precision: Precision::Exact,
                unit: Some("$".to_string()),
            }),
            range: Some(5..14),
            entity: "snips/amountOfMoney".to_string(),
            slot_name: "amount".to_string(),
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn should_resolve_custom_slot() {
        // Given
        let entity = Entity { automatically_extensible: false };
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
                entity_identifier: "userType".to_string(),
            },
            CustomEntity {
                value: "subscriber".to_string(),
                range: 27..37,
                resolved_value: "Subscriber".to_string(),
                entity_identifier: "userType".to_string(),
            }
        ];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(vec![]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot, &entity, &custom_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("Subscriber".into()),
            range: Some(27..37),
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn should_resolve_custom_slot_when_no_entities_found_on_whole_input() {
        // Given
        let entity = Entity { automatically_extensible: false };
        let internal_slot = InternalSlot {
            value: "subscriber".to_string(),
            char_range: 27..37,
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        };
        let custom_entities = vec![];
        let mocked_entity_parser = Arc::new(MockedCustomEntityParser::from_iter(
            vec![(
                "subscriber".to_string(),
                vec![
                    CustomEntity {
                        value: "subscriber".to_string(),
                        range: 0..10,
                        resolved_value: "Subscriber".to_string(),
                        entity_identifier: "userType".to_string(),
                    }
                ]
            )]));

        // When
        let resolved_slot = resolve_custom_slot(
            internal_slot, &entity, &custom_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("Subscriber".into()),
            range: Some(27..37),
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn should_resolve_custom_slot_when_automatically_extensible() {
        // Given
        let entity = Entity { automatically_extensible: true };
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
            internal_slot, &entity, &custom_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = Some(Slot {
            raw_value: "subscriber".to_string(),
            value: SlotValue::Custom("subscriber".into()),
            range: Some(27..37),
            entity: "userType".to_string(),
            slot_name: "userType".to_string(),
        });
        assert_eq!(expected_result, resolved_slot);
    }

    #[test]
    fn should_not_resolve_custom_slot_when_not_automatically_extensible() {
        // Given
        let entity = Entity { automatically_extensible: false };
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
            internal_slot, &entity, &custom_entities, mocked_entity_parser).unwrap();

        // Then
        let expected_result = None;
        assert_eq!(expected_result, resolved_slot);
    }
}
