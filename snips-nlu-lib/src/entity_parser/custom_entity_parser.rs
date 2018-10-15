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
use snips_nlu_ontology_parsers::{GazetteerEntityMatch, GazetteerParser};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CustomEntityParserUsage {
    WithStems,
    WithoutStems,
    WithAndWithoutStems,
}


impl CustomEntityParserUsage {
    pub fn from_u8(i: u8) -> Result<CustomEntityParserUsage> {
        match i {
            0 => Ok(CustomEntityParserUsage::WithStems),
            1 => Ok(CustomEntityParserUsage::WithoutStems),
            2 => Ok(CustomEntityParserUsage::WithAndWithoutStems),
            _ => bail!("Unknown parser usage identifier: {}", i),
        }
    }
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
pub struct CustomEntityParserMetadata {
    pub language: String,
    pub parser_directory: String,
    pub parser_usage: u8,
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
}
