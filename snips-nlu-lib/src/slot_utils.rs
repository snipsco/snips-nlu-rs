use std::ops::Range;

use snips_nlu_ontology::{BuiltinEntityKind, Slot, SlotValue};
use snips_nlu_ontology_parsers::BuiltinEntityParser;

#[derive(Debug, Clone, PartialEq)]
pub struct InternalSlot {
    pub value: String,
    pub char_range: Range<usize>,
    pub entity: String,
    pub slot_name: String,
}

pub fn convert_to_custom_slot(slot: InternalSlot) -> Slot {
    Slot {
        raw_value: slot.value.clone(),
        value: SlotValue::Custom(slot.value.into()),
        range: Some(slot.char_range),
        entity: slot.entity,
        slot_name: slot.slot_name,
    }
}

pub fn convert_to_builtin_slot(slot: InternalSlot, slot_value: SlotValue) -> Slot {
    Slot {
        raw_value: slot.value,
        value: slot_value,
        range: Some(slot.char_range),
        entity: slot.entity,
        slot_name: slot.slot_name,
    }
}

pub fn resolve_builtin_slots(
    text: &str,
    slots: Vec<InternalSlot>,
    parser: &BuiltinEntityParser,
    filter_entity_kinds: Option<&[BuiltinEntityKind]>,
) -> Vec<Slot> {
    let builtin_entities = parser.extract_entities(text, filter_entity_kinds);
    slots
        .into_iter()
        .filter_map(|slot| {
            if let Ok(entity_kind) = BuiltinEntityKind::from_identifier(&slot.entity) {
                builtin_entities
                    .iter()
                    .find(|entity| {
                        entity.entity_kind == entity_kind && entity.range == slot.char_range
                    })
                    .map(|rustling_entity| Some(rustling_entity.entity.clone()))
                    .unwrap_or({
                        parser
                            .extract_entities(&slot.value, Some(&[entity_kind]))
                            .into_iter()
                            .find(|rustling_entity| rustling_entity.entity_kind == entity_kind)
                            .map(|rustling_entity| rustling_entity.entity)
                    })
                    .map(|matching_entity| convert_to_builtin_slot(slot, matching_entity))
            } else {
                Some(convert_to_custom_slot(slot))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snips_nlu_ontology::{AmountOfMoneyValue, Language, OrdinalValue, Precision};

    #[test]
    fn resolve_builtin_slots_works() {
        // Given
        let text = "Send 5 dollars to the 10th subscriber";
        let slots = vec![
            InternalSlot {
                value: "5 dollars".to_string(),
                char_range: 5..14,
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "amount".to_string(),
            },
            InternalSlot {
                value: "10th".to_string(),
                char_range: 22..26,
                entity: "snips/ordinal".to_string(),
                slot_name: "ranking".to_string(),
            },
        ];
        let parser = BuiltinEntityParser::get(Language::EN);

        // When
        let filter_entity_kinds = &[BuiltinEntityKind::AmountOfMoney, BuiltinEntityKind::Ordinal];
        let actual_results =
            resolve_builtin_slots(text, slots, &*parser, Some(filter_entity_kinds));

        // Then
        let expected_results = vec![
            Slot {
                raw_value: "5 dollars".to_string(),
                value: SlotValue::AmountOfMoney(AmountOfMoneyValue {
                    value: 5.0,
                    precision: Precision::Exact,
                    unit: Some("$".to_string()),
                }),
                range: Some(5..14),
                entity: "snips/amountOfMoney".to_string(),
                slot_name: "amount".to_string(),
            },
            Slot {
                raw_value: "10th".to_string(),
                value: SlotValue::Ordinal(OrdinalValue { value: 10 }),
                range: Some(22..26),
                entity: "snips/ordinal".to_string(),
                slot_name: "ranking".to_string(),
            },
        ];
        assert_eq!(expected_results, actual_results);
    }
}
