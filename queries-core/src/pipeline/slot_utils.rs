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
            range,
            entity,
            slot_name
        }
    }
}

impl Slot {
    pub fn update_with_slot_value(&self, slot_value: SlotValue) -> Slot {
        Slot {
            raw_value: self.raw_value.clone(),
            value: slot_value,
            range: self.range.clone(),
            entity: self.entity.clone(),
            slot_name: self.slot_name.clone()
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
            SlotValue::Builtin(ref builtin_entity_value) => state.serialize_field("value", &builtin_entity_value)?,
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
        range: slot.range,
        entity: slot.entity,
        slot_name: slot.slot_name
    }
}

pub fn convert_to_builtin_slot(slot: InternalSlot, builtin_entity: BuiltinEntity) -> Slot {
    Slot {
        raw_value: slot.value,
        value: SlotValue::Builtin(builtin_entity),
        range: slot.range,
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
