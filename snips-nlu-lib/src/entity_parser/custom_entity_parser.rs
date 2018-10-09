use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use failure::ResultExt;
use itertools::Itertools;
use nlu_utils::language::Language as NluUtilsLanguage;
use nlu_utils::token::*;
use serde_json;
use snips_nlu_ontology::Language;
use snips_nlu_ontology_parsers::{
    GazetteerEntityMatch,
    GazetteerParser,
    GazetteerEntityParserBuilder,
    GazetteerParserBuilder
};
use snips_nlu_ontology_parsers::gazetteer_entity_parser::{
    ParserBuilder, EntityValue};

use entity_parser::utils::Cache;
use errors::*;
use language::FromLanguage;

use utils::EntityName;

pub type CustomEntity = GazetteerEntityMatch<String>;

pub trait CustomEntityParser: Send + Sync {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[String]>,
    ) -> Result<Vec<CustomEntity>>;
}

pub struct CachingCustomEntityParser {
    language: NluUtilsLanguage,
    parser: GazetteerParser<String>,
    cache: Mutex<Cache<CacheKey, Vec<CustomEntity>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    input: String,
    kinds: Vec<EntityName>,
}

impl CustomEntityParser for CachingCustomEntityParser {
    fn extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[String]>,
    ) -> Result<Vec<CustomEntity>> {
        let lowercased_sentence = sentence.to_lowercase();
        let cache_key = CacheKey {
            input: lowercased_sentence,
            kinds: filter_entity_kinds
                .map(|entity_kinds| entity_kinds.to_vec())
                .unwrap_or_else(|| vec![]),
        };

        self.cache
            .lock()
            .unwrap()
            .try_cache(&cache_key,
                       |cache_key| self._extract_entities(&cache_key.input, filter_entity_kinds))
    }
}

impl CachingCustomEntityParser {
    fn _extract_entities(
        &self,
        sentence: &str,
        filter_entity_kinds: Option<&[String]>,
    ) -> Result<Vec<CustomEntity>> {
        let tokens = tokenize(sentence, self.language);
        let shifts = compute_char_shifts(&tokens);
        let cleaned_input = tokens.into_iter().map(|token| token.value).join(" ");
        Ok(self.parser.extract_entities(&cleaned_input, filter_entity_kinds)?
            .into_iter()
            .map(|mut entity_match| {
                let range_start = entity_match.range.start;
                let range_end = entity_match.range.end;
                let remapped_range_start = (range_start as i32 - shifts[range_start]) as usize;
                let remapped_range_end = (range_end as i32 - shifts[range_end - 1]) as usize;
                entity_match.range = remapped_range_start..remapped_range_end;
                entity_match
            })
            .collect()
        )
    }
}

/// Compute the shifts in characters that occur when comparing the tokens string
/// with the string consisting of all tokens separated with a space
///
/// # Examples
///
/// For instance, if "hello?world" is tokenized in ["hello", "?", "world"],
/// then the character shifts between "hello?world" and "hello ? world" are
/// [0, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2]
fn compute_char_shifts(tokens: &Vec<Token>) -> Vec<i32> {
    if tokens.is_empty() {
        return vec![];
    }

    let mut characters_shifts = vec![];
    let mut current_shift = 0;

    for (token_index, token) in tokens.iter().enumerate() {
        let (previous_token_end, previous_space_len) = if token_index == 0 {
            (0, 0)
        } else {
            (tokens[token_index - 1].char_range.end as i32, 1)
        };
        current_shift -= (token.char_range.start as i32 - previous_token_end) - previous_space_len;
        let token_len = token.char_range.clone().count() as i32;
        let index_shift = token_len + previous_space_len;
        characters_shifts.extend((0..index_shift).map(|_| current_shift));
    }
    characters_shifts
}

#[derive(Deserialize)]
struct CustomEntityParserMetadata {
    language: String,
    parser_directory: String,
}

impl CachingCustomEntityParser {
    pub fn from_path<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
        let metadata_path = path.as_ref().join("metadata.json");
        let metadata_file = File::open(&metadata_path)
            .with_context(|_|
                format!("Cannot open metadata file for custom entity parser at path: {:?}",
                        metadata_path))?;
        let metadata: CustomEntityParserMetadata = serde_json::from_reader(metadata_file)
            .with_context(|_| "Cannot deserialize custom entity parser metadata")?;
        let language = NluUtilsLanguage::from_language(Language::from_str(&metadata.language)?);
        let gazetteer_parser_path = path.as_ref().join(&metadata.parser_directory);
        let parser = GazetteerParser::from_path(gazetteer_parser_path)?;
        let cache = Mutex::new(Cache::new(cache_capacity));
        Ok(Self { language, parser, cache })
    }
}

pub struct CachingCustomEntityParserBuilder {
    language: NluUtilsLanguage,
    entity_values: HashMap<String, Vec<EntityValue>>,
    entity_parser_thresholds: HashMap<String, f32>,
    cache_capacity: usize,
}

impl CachingCustomEntityParserBuilder {

    pub fn new(language: NluUtilsLanguage, cache_capacity: usize) -> Self {
        CachingCustomEntityParserBuilder {
            language,
            entity_values: HashMap::new(),
            entity_parser_thresholds: HashMap::new(),
            cache_capacity,
        }
    }

    pub fn build(self) -> Result<CachingCustomEntityParser> {
        let cache = Mutex::new(Cache::new(self.cache_capacity));

        let gazetteer_entity_parser_builders: Vec<GazetteerEntityParserBuilder> = self.entity_values
            .iter()
            .map(|(entity, entity_values)| {
                let mut parser_builder = ParserBuilder::default();
                if let Some(ratio) = self.entity_parser_thresholds.get(&*entity) {
                    parser_builder = parser_builder.minimum_tokens_ratio(*ratio);
                }
                for value in entity_values.into_iter() {
                    parser_builder = parser_builder.add_value(value.clone());
                }
                GazetteerEntityParserBuilder {
                    entity_identifier: entity.clone(),
                    entity_parser: parser_builder,
                }
            })
            .collect();

        let gazetteer_parser_builder = GazetteerParserBuilder {
            entity_parsers: gazetteer_entity_parser_builders
        };

        let parser = gazetteer_parser_builder.build()?;
        Ok(
            CachingCustomEntityParser {
                language: self.language,
                parser,
                cache,
            }
        )
    }

    pub fn add_value(mut self, entity: String, entity_value: EntityValue) -> Self {
        if !self.entity_values.contains_key(&*entity) {
           self.entity_values.insert(entity.clone(), vec![]);
        }
        self.entity_values
            .get_mut(&*entity)
            .map(|values| values.push(entity_value));
        self
    }

    pub fn minimum_tokens_ratio(mut self, entity: String, ratio: f32) -> Self {
        self.entity_parser_thresholds.insert(entity, ratio);
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use testutils::file_path;

    #[test]
    fn should_compute_char_shifts() {
        // Given
        let tokens = vec![
            Token::new(
                "hello".to_string(),
                0..5,
                0..5,
            ),
            Token::new(
                "?".to_string(),
                5..6,
                5..6,
            ),
            Token::new(
                "world".to_string(),
                6..11,
                6..11,
            )
        ];

        // When / Then
        assert_eq!(vec![0, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2], compute_char_shifts(&tokens));
    }

    #[test]
    fn custom_entity_parser_should_handle_char_shifts() {
        // Given
        let parser_path = file_path("tests")
            .join("models")
            .join("nlu_engine")
            .join("custom_entity_parser");

        let custom_entity_parser = CachingCustomEntityParser::from_path(parser_path, 1000).unwrap();
        let input = "Make me a  ?hot tea";

        // When
        let entities = custom_entity_parser.extract_entities(input, None).unwrap();

        // Then
        let expected_entities = vec![
            CustomEntity {
                value: "hot".to_string(),
                resolved_value: "hot".to_string(),
                range: 12..15,
                entity_identifier: "Temperature".to_string(),
            }
        ];

        assert_eq!(expected_entities, entities);
    }

    #[test]
    fn should_build_custom_entity_parser() {
        // Given
        let language = NluUtilsLanguage::EN;

        let entity_1 = "entity_1";
        let entity_2 = "entity_2";

        let value_1 = EntityValue {
            raw_value: "dummy_1".to_string(),
            resolved_value: "dummy_1_resolved".to_string(),
        };

        let value_2 = EntityValue {
            raw_value: "dummy_2".to_string(),
            resolved_value: "dummy_2_resolved".to_string(),
        };
        let value_3 = EntityValue {
            raw_value: "dummy_3".to_string(),
            resolved_value: "dummy_3_resolved".to_string(),
        };


        // When
        let mut custom_gazetteer_parser_builder = CachingCustomEntityParserBuilder::new(language, cache_capacity);
        custom_gazetteer_parser_builder = custom_gazetteer_parser_builder
            .add_value(entity_1.to_string(), value_1);
        custom_gazetteer_parser_builder = custom_gazetteer_parser_builder
            .add_value(entity_2.to_string(), value_2);
        custom_gazetteer_parser_builder = custom_gazetteer_parser_builder
            .add_value(entity_2.to_string(), value_3);

        custom_gazetteer_parser_builder = custom_gazetteer_parser_builder
            .minimum_tokens_ratio(entity_1.to_string(), 0.7);

        let parser = custom_gazetteer_parser_builder.build().unwrap();

        // Then
        let result_1 = parser.extract_entities("dummy_1", None).unwrap();
        assert_eq!(
            result_1,
            vec![
                GazetteerEntityMatch {
                    value: "dummy_1".to_string(),
                    resolved_value: "dummy_1_resolved".to_string(),
                    entity_identifier: entity_1.to_string(),
                    range: 0..7,
                }
            ]
        );

        let result_2 = parser.extract_entities("dummy_2", None).unwrap();
        assert_eq!(
            result_2,
            vec![
                GazetteerEntityMatch {
                    value: "dummy_2".to_string(),
                    resolved_value: "dummy_2_resolved".to_string(),
                    entity_identifier: entity_2.to_string(),
                    range: 0..7,
                }
            ]
        );

        let result_3 = parser.extract_entities("dummy_3", None).unwrap();
        assert_eq!(
            result_3,
            vec![
                GazetteerEntityMatch {
                    value: "dummy_3".to_string(),
                    resolved_value: "dummy_3_resolved".to_string(),
                    entity_identifier: entity_2.to_string(),
                    range: 0..7,
                }
            ]
        );
    }
}
