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
    use snips_nlu_ontology::{AmountOfMoneyValue, Language, OrdinalValue, Precision};
    use std::collections::HashMap;
    use models::nlu_engine::Entity;
    use models::nlu_engine::DatasetMetadata;

//    #[test]
//    fn resolve_slots_works() {
//        // Given
//        let text = "Send 5 dollars to the 10th subscriber";
//        let slots = vec![
//            InternalSlot {
//                value: "5 dollars".to_string(),
//                char_range: 5..14,
//                entity: "snips/amountOfMoney".to_string(),
//                slot_name: "amount".to_string(),
//            },
//            InternalSlot {
//                value: "10th".to_string(),
//                char_range: 22..26,
//                entity: "snips/ordinal".to_string(),
//                slot_name: "ranking".to_string(),
//            },
//            InternalSlot {
//                value: "subscriber".to_string(),
//                char_range: 27..37,
//                entity: "userType".to_string(),
//                slot_name: "userType".to_string(),
//            }
//        ];
//        let parser = CachingBuiltinEntityParser::from_language(Language::EN, 1000).unwrap();
//        let entity = Entity {
//            automatically_extensible: true,
//        };
//        let entities = [("userType".to_string(), entity)].iter().cloned().collect();
//        let dataset_metadata = DatasetMetadata {
//            language_code: Language::EN.to_string(),
//            entities,
//            slot_name_mappings: HashMap::new(),
//        };
//
//        // When
//        let filter_entity_kinds = &[BuiltinEntityKind::AmountOfMoney, BuiltinEntityKind::Ordinal];
//        let actual_results = resolve_slots(
//            text, slots, &dataset_metadata, &parser, Some(filter_entity_kinds));
//
//        // Then
//        let expected_results = vec![
//            Slot {
//                raw_value: "5 dollars".to_string(),
//                value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
//                    value: 5.0,
//                    precision: Precision::Exact,
//                    unit: Some("$".to_string()),
//                }),
//                range: Some(5..14),
//                entity: "snips/amountOfMoney".to_string(),
//                slot_name: "amount".to_string(),
//            },
//            Slot {
//                raw_value: "10th".to_string(),
//                value: SlotValue::Ordinal(OrdinalValue { value: 10 }),
//                range: Some(22..26),
//                entity: "snips/ordinal".to_string(),
//                slot_name: "ranking".to_string(),
//            },
//            Slot {
//                raw_value: "subscriber".to_string(),
//                value: SlotValue::Custom("member".to_string().into()),
//                range: Some(27..37),
//                entity: "userType".to_string(),
//                slot_name: "userType".to_string(),
//            }
//        ];
//        assert_eq!(expected_results, actual_results);
//    }
}
