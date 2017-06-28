use builtin_entities::{RustlingParser, BuiltinEntityKind};
use core_ontology::{BuiltinEntity, Slot, SlotValue};
use pipeline::InternalSlot;

pub fn convert_to_custom_slot(slot: InternalSlot) -> Slot {
    Slot {
        raw_value: slot.value.clone(),
        value: SlotValue::Custom(slot.value),
        range: Some(slot.range),
        entity: slot.entity,
        slot_name: slot.slot_name
    }
}

pub fn convert_to_builtin_slot(slot: InternalSlot, builtin_entity: BuiltinEntity) -> Slot {
    Slot {
        raw_value: slot.value,
        value: SlotValue::Builtin(builtin_entity),
        range: Some(slot.range),
        entity: slot.entity,
        slot_name: slot.slot_name,
    }
}

pub fn resolve_builtin_slots(text: &str, slots: Vec<InternalSlot>, parser: &RustlingParser) -> Vec<Slot> {
    let builtin_entities = parser.extract_entities(text, None);
    slots.into_iter()
        .filter_map(|slot|
            if let Some(entity_kind) = BuiltinEntityKind::from_identifier(&slot.entity).ok() {
                builtin_entities.iter()
                    .find(|entity| entity.entity_kind == entity_kind && entity.range == slot.range)
                    .map(|rustling_entity| Some(rustling_entity.entity.clone()))
                    .unwrap_or({
                        parser.extract_entities(&slot.value, None).into_iter()
                            .find(|rustling_entity| rustling_entity.entity_kind == entity_kind)
                            .map(|rustling_entity| rustling_entity.entity)
                    })
                    .map(|matching_entity| convert_to_builtin_slot(slot, matching_entity))
            } else {
                Some(convert_to_custom_slot(slot))
            })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use rustling_ontology::Lang;
    use builtin_entities::{AmountOfMoneyValue, OrdinalValue, Precision};

    #[test]
    fn resolve_builtin_slots_works() {
        // Given
        let text = "Send 5 dollars to the 10th subscriber";
        let slots = vec![
            InternalSlot {
                value: "5 dollars".to_string(),
                range: 5..14,
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "amount".to_string()
            },
            InternalSlot {
                value: "10th".to_string(),
                range: 22..26,
                entity: "snips/ordinal".to_string(),
                slot_name: "ranking".to_string()
            },
        ];
        let parser = RustlingParser::get(Lang::EN);

        // When
        let actual_results = resolve_builtin_slots(text, slots, &*parser);

        // Then
        let expected_results = vec![
            Slot {
                raw_value: "5 dollars".to_string(),
                value: SlotValue::Builtin(BuiltinEntity::AmountOfMoney(
                    AmountOfMoneyValue {
                        value: 5.0,
                        precision: Precision::Exact,
                        unit: Some("$".to_string())
                    }
                )),
                range: Some(5..14),
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "amount".to_string()
            },
            Slot {
                raw_value: "10th".to_string(),
                value: SlotValue::Builtin(BuiltinEntity::Ordinal(OrdinalValue(10))),
                range: Some(22..26),
                entity: "snips/ordinal".to_string(),
                slot_name: "ranking".to_string()
            }
        ];
        assert_eq!(expected_results, actual_results);
    }
}
