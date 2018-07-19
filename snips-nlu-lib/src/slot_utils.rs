use std::ops::Range;

use builtin_entity_parsing::CachingBuiltinEntityParser;
use models::nlu_engine::DatasetMetadata;
use snips_nlu_ontology::{BuiltinEntityKind, Slot, SlotValue};
use nlu_utils::string::normalize;
use utils::{EntityName, SlotName};

#[derive(Debug, Clone, PartialEq)]
pub struct InternalSlot {
    pub value: String,
    pub char_range: Range<usize>,
    pub entity: EntityName,
    pub slot_name: SlotName,
}

fn convert_to_custom_slot(
    slot: InternalSlot,
    opt_resolved_value: Option<String>,
) -> Slot {
    let value = opt_resolved_value
        .map(|resolved_value| SlotValue::Custom(resolved_value.into()))
        .unwrap_or_else(|| SlotValue::Custom(slot.value.clone().into()));
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

pub fn resolve_slots(
    text: &str,
    slots: Vec<InternalSlot>,
    dataset_metadata: &DatasetMetadata,
    parser: &CachingBuiltinEntityParser,
    filter_entity_kinds: Option<&[BuiltinEntityKind]>,
) -> Vec<Slot> {
    let builtin_entities = parser.extract_entities(text, filter_entity_kinds, false);
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
                            .extract_entities(&slot.value, Some(&[entity_kind]), false)
                            .into_iter()
                            .find(|rustling_entity| rustling_entity.entity_kind == entity_kind)
                            .map(|rustling_entity| rustling_entity.entity)
                    })
                    .map(|matching_entity| convert_to_builtin_slot(slot, matching_entity))
            } else {
                dataset_metadata.entities.get(&slot.entity)
                    .map(|entity|
                        entity
                            .utterances
                            .get(&slot.value)
                            .or(entity.utterances.get(&normalize(&slot.value)))
                            .map(|reference_value|
                                convert_to_custom_slot(slot.clone(), Some(reference_value.clone())))
                            .or_else(|| if entity.automatically_extensible {
                                Some(convert_to_custom_slot(slot, None))
                            } else {
                                None
                            })
                    )
                    .unwrap()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snips_nlu_ontology::{AmountOfMoneyValue, Language, OrdinalValue, Precision};
    use std::collections::HashMap;
    use models::nlu_engine::Entity;

    #[test]
    fn resolve_slots_works() {
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
            InternalSlot {
                value: "subscriber".to_string(),
                char_range: 27..37,
                entity: "userType".to_string(),
                slot_name: "userType".to_string(),
            }
        ];
        let parser = CachingBuiltinEntityParser::from_language(Language::EN, 1000).unwrap();
        let entity = Entity {
            automatically_extensible: true,
            utterances: [("subscriber".to_string(), "member".to_string())].iter().cloned().collect(),
        };
        let entities = [("userType".to_string(), entity)].iter().cloned().collect();
        let dataset_metadata = DatasetMetadata {
            language_code: Language::EN.to_string(),
            entities,
            slot_name_mappings: HashMap::new(),
        };

        // When
        let filter_entity_kinds = &[BuiltinEntityKind::AmountOfMoney, BuiltinEntityKind::Ordinal];
        let actual_results = resolve_slots(
            text, slots, &dataset_metadata, &parser, Some(filter_entity_kinds));

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
            Slot {
                raw_value: "subscriber".to_string(),
                value: SlotValue::Custom("member".to_string().into()),
                range: Some(27..37),
                entity: "userType".to_string(),
                slot_name: "userType".to_string()
            }
        ];
        assert_eq!(expected_results, actual_results);
    }
}
