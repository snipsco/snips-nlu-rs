use std::result::Result as StdResult;
use std::ops::Range;

use serde::ser::{Serialize, Serializer, SerializeStruct};

use builtin_entities::{RustlingParser, BuiltinEntity, BuiltinEntityKind};
use pipeline::{InternalSlot, Slot, SlotValue};

impl Slot {
    pub fn new_custom(value: String, range: Range<usize>, entity: String, slot_name: String) -> Slot {
        Slot {
            raw_value: value.clone(),
            value: SlotValue::Custom(value),
            range: Some(range),
            entity,
            slot_name
        }
    }
}

impl Slot {
    pub fn with_slot_value(self, slot_value: SlotValue) -> Slot {
        Slot {
            raw_value: self.raw_value,
            value: slot_value,
            range: self.range,
            entity: self.entity,
            slot_name: self.slot_name
        }
    }
}

impl Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("Slot", 4)?;
        match self.value {
            SlotValue::Custom(ref string_value) => state.serialize_field("value", &string_value)?,
            SlotValue::Builtin(ref builtin_entity_value) => {
                match builtin_entity_value {
                    &BuiltinEntity::Number(ref v) => state.serialize_field("value", &v)?,
                    &BuiltinEntity::Ordinal(ref v) => state.serialize_field("value", &v)?,
                    &BuiltinEntity::Time(ref v) => state.serialize_field("value", &v)?,
                    &BuiltinEntity::AmountOfMoney(ref v) => state.serialize_field("value", &v)?,
                    &BuiltinEntity::Temperature(ref v) => state.serialize_field("value", &v)?,
                    &BuiltinEntity::Duration(ref v) => state.serialize_field("value", &v)?,
                }
            },
        }
        state.serialize_field("range", &self.range)?;
        state.serialize_field("entity", &self.entity)?;
        state.serialize_field("slot_name", &self.slot_name)?;
        state.end()
    }
}

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
                        unit: Some("$")
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
