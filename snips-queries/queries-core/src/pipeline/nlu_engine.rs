use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;
use itertools::Itertools;

use builtin_entities::{BuiltinEntityKind, RustlingParser};
use errors::*;
use snips_queries_ontology::{IntentParserResult, Slot, SlotValue, TaggedEntity};
use pipeline::IntentParser;
use pipeline::rule_based::RuleBasedIntentParser;
use pipeline::probabilistic::ProbabilisticIntentParser;
use pipeline::tagging_utils::{disambiguate_tagged_entities, enrich_entities, tag_builtin_entities};
use pipeline::configuration::{Entity, NluEngineConfigurationConvertible};
use rustling_ontology::Lang;
use language::LanguageConfig;
use nlu_utils::language::Language;
use nlu_utils::token::{compute_all_ngrams, tokenize};
use nlu_utils::string::{normalize, substring_with_char_range};

const MODEL_VERSION: &str = "0.11.0";

pub struct SnipsNluEngine {
    language_config: LanguageConfig,
    parsers: Vec<Box<IntentParser>>,
    entities: HashMap<String, Entity>,
    intents_data_sizes: HashMap<String, usize>,
    slot_name_mapping: HashMap<String, HashMap<String, String>>,
    builtin_entity_parser: Option<Arc<RustlingParser>>,
}

impl SnipsNluEngine {
    pub fn new<T: NluEngineConfigurationConvertible + 'static>(configuration: T) -> Result<Self> {
        let nlu_config = configuration.into_nlu_engine_configuration();

        let mut parsers: Vec<Box<IntentParser>> = Vec::with_capacity(2);

        let model = nlu_config.model;
        if let Some(config) = model.rule_based_parser {
            parsers.push(Box::new(RuleBasedIntentParser::new(config)
                .chain_err(|| "Can't create RuleBasedIntentParser")?))
        };
        if let Some(config) = model.probabilistic_parser {
            parsers.push(Box::new(ProbabilisticIntentParser::new(config)
                .chain_err(|| "Can't create ProbabilisticIntentParser")?))
        };
        let intents_data_sizes = nlu_config.intents_data_sizes;
        let slot_name_mapping = nlu_config.slot_name_mapping;
        let builtin_entity_parser = Lang::from_str(&nlu_config.language)
            .ok()
            .map(|rustling_lang| RustlingParser::get(rustling_lang));
        let language_config = LanguageConfig::from_str(&nlu_config.language)?;

        Ok(SnipsNluEngine {
            language_config: language_config,
            parsers,
            entities: nlu_config.entities,
            intents_data_sizes,
            slot_name_mapping,
            builtin_entity_parser,
        })
    }

    pub fn parse(
        &self,
        input: &str,
        intents_filter: Option<&[String]>,
    ) -> Result<IntentParserResult> {
        if self.parsers.is_empty() {
            return Ok(IntentParserResult {
                input: input.to_string(),
                intent: None,
                slots: None,
            });
        }
        let set_intents: Option<HashSet<String>> = intents_filter.map(|intent_list| {
            HashSet::from_iter(intent_list.iter().map(|name| name.to_string()))
        });

        for parser in self.parsers.iter() {
            let classification_result = parser.get_intent(input, set_intents.as_ref())?;
            if let Some(classification_result) = classification_result {
                let valid_slots = parser
                    .get_slots(input, &classification_result.intent_name)?
                    .into_iter()
                    .filter_map(|slot| {
                        if let Some(entity) = self.entities.get(&slot.entity) {
                            entity
                                .utterances
                                .get(&slot.raw_value)
                                .map(|reference_value| {
                                    Some(slot.clone().with_slot_value(
                                        SlotValue::Custom(reference_value.to_string().into()),
                                    ))
                                })
                                .unwrap_or(if entity.automatically_extensible {
                                    Some(slot)
                                } else {
                                    None
                                })
                        } else {
                            Some(slot)
                        }
                    })
                    .collect();

                return Ok(IntentParserResult {
                    input: input.to_string(),
                    intent: Some(classification_result),
                    slots: Some(valid_slots),
                });
            }
        }
        Ok(IntentParserResult {
            input: input.to_string(),
            intent: None,
            slots: None,
        })
    }

    // TODO: Expose directly a static variable
    pub fn model_version() -> &'static str {
        MODEL_VERSION
    }
}

impl SnipsNluEngine {
    pub fn extract_slot(
        &self,
        input: String,
        intent_name: &str,
        slot_name: String,
    ) -> Result<Option<Slot>> {
        let entity_name = self.slot_name_mapping
            .get(intent_name)
            .ok_or(format!("Unknown intent: {}", intent_name))?
            .get(&slot_name)
            .ok_or(format!("Unknown slot: {}", &slot_name))?;

        let slot = if let Some(custom_entity) = self.entities.get(entity_name) {
            extract_custom_slot(
                input,
                entity_name.to_string(),
                slot_name.to_string(),
                custom_entity.clone(),
                self.language_config.language,
            )
        } else {
            if let Some(builtin_entity_parser) = self.builtin_entity_parser.clone() {
                extract_builtin_slot(
                    input,
                    entity_name.to_string(),
                    slot_name.to_string(),
                    builtin_entity_parser,
                )?
            } else {
                None
            }
        };
        Ok(slot)
    }
}

fn extract_custom_slot(
    input: String,
    entity_name: String,
    slot_name: String,
    custom_entity: Entity,
    language: Language,
) -> Option<Slot> {
    let tokens = tokenize(&input, language);
    let token_values_ref = tokens.iter().map(|v| &*v.value).collect_vec();
    let mut ngrams = compute_all_ngrams(&*token_values_ref, tokens.len());
    ngrams.sort_by_key(|&(_, ref indexes)| -(indexes.len() as i16));

    ngrams
        .into_iter()
        .find(|&(ref ngram, _)| {
            custom_entity.utterances.contains_key(&normalize(&ngram))
        })
        .map(|(ngram, _)| {
            Some(Slot {
                raw_value: ngram.clone(),
                value: SlotValue::Custom(
                    custom_entity
                        .utterances
                        .get(&normalize(&ngram))
                        .unwrap()
                        .to_string()
                        .into(),
                ),
                range: None,
                entity: entity_name.clone(),
                slot_name: slot_name.clone(),
            })
        })
        .unwrap_or(if custom_entity.automatically_extensible {
            Some(Slot {
                raw_value: input.clone(),
                value: SlotValue::Custom(input.into()),
                range: None,
                entity: entity_name,
                slot_name: slot_name,
            })
        } else {
            None
        })
}

fn extract_builtin_slot(
    input: String,
    entity_name: String,
    slot_name: String,
    builtin_entity_parser: Arc<RustlingParser>,
) -> Result<Option<Slot>> {
    let builtin_entity_kind = BuiltinEntityKind::from_identifier(&entity_name)?;
    Ok(
        builtin_entity_parser
            .extract_entities(&input, Some(&[builtin_entity_kind]))
            .first()
            .map(|rustlin_entity| {
                Slot {
                    raw_value: substring_with_char_range(input, &rustlin_entity.range),
                    value: rustlin_entity.entity.clone(),
                    range: None,
                    entity: entity_name,
                    slot_name: slot_name,
                }
            }),
    )
}

const DEFAULT_THRESHOLD: usize = 5;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PartialTaggedEntity {
    pub value: String,
    pub range: Option<Range<usize>>,
    pub entity: String,
    pub slot_name: Option<String>,
}

impl PartialTaggedEntity {
    fn into_tagged_entity(self) -> Option<TaggedEntity> {
        if let Some(slot_name) = self.slot_name {
            Some(TaggedEntity {
                value: self.value,
                range: self.range,
                entity: self.entity,
                slot_name,
            })
        } else {
            None
        }
    }
}

impl SnipsNluEngine {
    pub fn tag(
        &self,
        text: &str,
        intent: &str,
        small_data_regime_threshold: Option<usize>,
    ) -> Result<Vec<TaggedEntity>> {
        let intent_data_size: usize = *self.intents_data_sizes
            .get(intent)
            .ok_or(format!("Unknown intent: {}", intent))?;
        let slot_name_mapping = self.slot_name_mapping
            .get(intent)
            .ok_or(format!("Unknown intent: {}", intent))?;
        let intent_entities = HashSet::from_iter(slot_name_mapping.values());
        let threshold = small_data_regime_threshold.unwrap_or(DEFAULT_THRESHOLD);
        if intent_data_size >= threshold {
            Ok(
                self.parse(text, Some(&[intent.into()]))?
                    .slots
                    .map(|slots| {
                        slots
                            .into_iter()
                            .map(|s| {
                                TaggedEntity {
                                    value: s.raw_value,
                                    range: s.range,
                                    entity: s.entity,
                                    slot_name: s.slot_name,
                                }
                            })
                            .collect_vec()
                    })
                    .unwrap_or(vec![]),
            )
        } else {
            let tagged_seen_entities = self.tag_seen_entities(text, intent_entities, self.language_config.language);
            let tagged_builtin_entities = tag_builtin_entities(text, self.language_config.language);
            let tagged_entities = enrich_entities(tagged_seen_entities, tagged_builtin_entities);
            let disambiguated_entities =
                disambiguate_tagged_entities(tagged_entities, slot_name_mapping.clone());
            Ok(
                disambiguated_entities
                    .into_iter()
                    .filter_map(|e| e.into_tagged_entity())
                    .collect(),
            )
        }
    }

    fn tag_seen_entities(
        &self,
        text: &str,
        intent_entities: HashSet<&String>,
        language: Language
    ) -> Vec<PartialTaggedEntity> {
        let entities = self.entities
            .clone()
            .into_iter()
            .filter_map(|(entity_name, entity)| {
                if intent_entities.contains(&entity_name) {
                    Some((entity_name, entity))
                } else {
                    None
                }
            })
            .collect_vec();
        let tokens = tokenize(text, language);
        let token_values_ref = tokens.iter().map(|v| &*v.value).collect_vec();
        let mut ngrams = compute_all_ngrams(&*token_values_ref, tokens.len());
        ngrams.sort_by_key(|&(_, ref indexes)| -(indexes.len() as i16));
        let mut tagged_entities = Vec::<PartialTaggedEntity>::new();
        for (ngram, ngram_indexes) in ngrams {
            let mut ngram_entity: Option<PartialTaggedEntity> = None;
            for &(ref entity_name, ref entity_data) in entities.iter() {
                if entity_data.utterances.contains_key(&normalize(&ngram)) {
                    if ngram_entity.is_some() {
                        // If the ngram matches several entities, i.e. there is some ambiguity, we
                        // don't add it to the tagged entities
                        ngram_entity = None;
                        break;
                    }
                    if let (Some(first), Some(last)) = (ngram_indexes.first(), ngram_indexes.last()) {
                        let range_start = tokens[*first].char_range.start;
                        let range_end = tokens[*last].char_range.end;
                        let range = range_start..range_end;
                        let value = substring_with_char_range(text.to_string(), &range);
                        ngram_entity = Some(PartialTaggedEntity {
                            value,
                            range: Some(range),
                            entity: entity_name.to_string(),
                            slot_name: None,
                        })
                    }
                }
            }
            if let Some(ngram_entity) = ngram_entity {
                tagged_entities = enrich_entities(tagged_entities, vec![ngram_entity])
            }
        }
        tagged_entities
    }
}

pub mod deprecated {
    #[deprecated(since = "0.21.0", note = "please use `SnipsNluEngine` instead")]
    pub type SnipsNLUEngine = super::SnipsNluEngine;
}

#[cfg(test)]
mod tests {
    use super::*;
    use snips_queries_ontology::{IntentClassifierResult, NumberValue};
    use pipeline::configuration::NluEngineConfiguration;
    use testutils::parse_json;

    #[test]
    fn parse_works() {
        // Given
        let configuration: NluEngineConfiguration =
            parse_json("tests/configurations/trained_assistant.json");
        let nlu_engine = SnipsNluEngine::new(configuration).unwrap();

        // When
        let result = nlu_engine
            .parse("Make me two cups of coffee please", None)
            .unwrap();

        // Then
        let expected_entity_value = SlotValue::Number(NumberValue { value: 2.0 });
        let expected_result = IntentParserResult {
            input: "Make me two cups of coffee please".to_string(),
            intent: Some(IntentClassifierResult {
                intent_name: "MakeCoffee".to_string(),
                probability: 0.7114164,
            }),
            slots: Some(vec![
                Slot {
                    raw_value: "two".to_string(),
                    value: expected_entity_value,
                    range: Some(8..11),
                    entity: "snips/number".to_string(),
                    slot_name: "number_of_cups".to_string(),
                },
            ]),
        };

        assert_eq!(expected_result, result)
    }

    #[test]
    fn tag_works_above_intent_data_threshold() {
        // Given
        let configuration: NluEngineConfiguration =
            parse_json("tests/configurations/trained_assistant.json");
        let nlu_engine = SnipsNluEngine::new(configuration).unwrap();
        let intent_data_threshold = 0;

        // When
        let tagged_entities = nlu_engine
            .tag(
                "Make me two cups of coffee please",
                "MakeCoffee",
                Some(intent_data_threshold),
            )
            .unwrap();

        // Then
        let expected_tagged_entities: Vec<TaggedEntity> = vec![
            TaggedEntity {
                value: "two".to_string(),
                range: Some(8..11),
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            },
        ];
        assert_eq!(expected_tagged_entities, tagged_entities)
    }

    #[test]
    fn tag_works_below_intent_data_threshold() {
        // Given
        let configuration: NluEngineConfiguration =
            parse_json("tests/configurations/trained_assistant.json");
        let nlu_engine = SnipsNluEngine::new(configuration).unwrap();
        let intent_data_threshold = 1000;

        // When
        let tagged_entities = nlu_engine
            .tag(
                "I want two hot cups of tea !!",
                "MakeTea",
                Some(intent_data_threshold),
            )
            .unwrap();

        // Then
        let expected_tagged_entities: Vec<TaggedEntity> = vec![
            TaggedEntity {
                value: "hot".to_string(),
                range: Some(11..14),
                entity: "Temperature".to_string(),
                slot_name: "beverage_temperature".to_string(),
            },
            TaggedEntity {
                value: "two".to_string(),
                range: Some(7..10),
                entity: "snips/number".to_string(),
                slot_name: "number_of_cups".to_string(),
            },
        ];
        assert_eq!(expected_tagged_entities, tagged_entities)
    }

    #[test]
    fn should_extract_custom_slot_when_tagged() {
        // Given
        let language = Language::EN;
        let input = "hello a b c d world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
            utterances: hashmap! {
                "a".to_string() => "value1".to_string(),
                "a b".to_string() => "value1".to_string(),
                "b c d".to_string() => "value2".to_string(),
            },
        };

        // When
        let extracted_slot = extract_custom_slot(input, entity_name, slot_name, custom_entity, language);

        // Then
        let expected_slot = Some(Slot {
            raw_value: "b c d".to_string(),
            value: SlotValue::Custom("value2".to_string().into()),
            range: None,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_extract_custom_slot_when_not_tagged() {
        // Given
        let language = Language::EN;
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: true,
            utterances: hashmap! {},
        };

        // When
        let extracted_slot = extract_custom_slot(input, entity_name, slot_name, custom_entity, language);

        // Then
        let expected_slot = Some(Slot {
            raw_value: "hello world".to_string(),
            value: SlotValue::Custom("hello world".to_string().into()),
            range: None,
            entity: "entity".to_string(),
            slot_name: "slot".to_string(),
        });
        assert_eq!(expected_slot, extracted_slot);
    }

    #[test]
    fn should_not_extract_custom_slot_when_not_extensible() {
        // Given
        let language = Language::EN;
        let input = "hello world".to_string();
        let entity_name = "entity".to_string();
        let slot_name = "slot".to_string();
        let custom_entity = Entity {
            automatically_extensible: false,
            utterances: hashmap! {},
        };

        // When
        let extracted_slot = extract_custom_slot(input, entity_name, slot_name, custom_entity, language);

        // Then
        let expected_slot = None;
        assert_eq!(expected_slot, extracted_slot);
    }
}

