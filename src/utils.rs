use std::collections::HashMap;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use failure::{format_err, ResultExt};
use zip::ZipArchive;

use snips_nlu_ontology::BuiltinEntity;
use snips_nlu_utils::range::ranges_overlap;
use snips_nlu_utils::string::{substring_with_char_range, suffix_from_char_index};

use crate::entity_parser::custom_entity_parser::CustomEntity;
use crate::errors::*;

pub type IntentName = String;
pub type SlotName = String;
pub type EntityName = String;

pub trait IterOps<T, I>: IntoIterator<Item = T>
    where I: IntoIterator<Item = T>,
          T: PartialEq {
    fn intersect(self, other: I) -> Vec<T>;
}

impl<T, I> IterOps<T, I> for I
    where I: IntoIterator<Item = T>,
          T: PartialEq
{
    fn intersect(self, other: I) -> Vec<T> {
        let v_other: Vec<_> = other.into_iter().collect();
        self.into_iter()
            .filter(|e1| v_other.iter().any(|e2| e1 == e2))
            .collect()
    }
}

pub fn deduplicate_overlapping_items<I, O, S, K>(
    items: Vec<I>,
    overlap: O,
    sort_key_fn: S,
) -> Vec<I>
where
    I: Clone,
    O: Fn(&I, &I) -> bool,
    S: FnMut(&I) -> K,
    K: Ord,
{
    let mut sorted_items = items.clone();
    sorted_items.sort_by_key(sort_key_fn);
    let mut deduplicated_items: Vec<I> = Vec::with_capacity(items.len());
    for item in sorted_items {
        if !deduplicated_items
            .iter()
            .any(|dedup_item| overlap(dedup_item, &item))
        {
            deduplicated_items.push(item);
        }
    }
    deduplicated_items
}

pub fn extract_nlu_engine_zip_archive<R: io::Read + io::Seek>(
    zip_reader: R,
    dest_path: &Path,
) -> Result<PathBuf> {
    let mut archive =
        ZipArchive::new(zip_reader).with_context(|_| "Could not read nlu engine zip data")?;
    for file_index in 0..archive.len() {
        let mut file = archive.by_index(file_index)?;
        let outpath = dest_path.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') || (&*file.name()).ends_with('\\') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile)?;
        }
    }
    let first_archive_file = archive.by_index(0)?.sanitized_name();
    let engine_dir_path = first_archive_file
        .components()
        .find(|component| {
            if let Component::Normal(_) = component {
                true
            } else {
                false
            }
        })
        .ok_or_else(|| format_err!("Trained engine archive is incorrect"))?
        .as_os_str();
    let engine_dir_name = engine_dir_path
        .to_str()
        .ok_or_else(|| format_err!("Engine directory name is empty"))?;
    Ok(dest_path.join(engine_dir_name))
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchedEntity {
    pub range: Range<usize>,
    pub entity_name: String,
}

impl Into<MatchedEntity> for BuiltinEntity {
    fn into(self) -> MatchedEntity {
        MatchedEntity {
            range: self.range,
            entity_name: self.entity_kind.identifier().to_string(),
        }
    }
}

impl Into<MatchedEntity> for CustomEntity {
    fn into(self) -> MatchedEntity {
        MatchedEntity {
            range: self.range,
            entity_name: self.entity_identifier,
        }
    }
}

pub fn replace_entities<F>(
    text: &str,
    matched_entities: Vec<MatchedEntity>,
    placeholder_fn: F,
) -> (HashMap<Range<usize>, Range<usize>>, String)
where
    F: Fn(&str) -> String,
{
    if matched_entities.is_empty() {
        return (HashMap::new(), text.to_string());
    }

    let mut dedup_matches = deduplicate_overlapping_entities(matched_entities);
    dedup_matches.sort_by_key(|entity| entity.range.start);

    let mut range_mapping: HashMap<Range<usize>, Range<usize>> = HashMap::new();
    let mut processed_text = "".to_string();
    let mut offset = 0;
    let mut current_ix = 0;

    for matched_entity in dedup_matches {
        let range_start = (matched_entity.range.start as i16 + offset) as usize;
        let prefix_text =
            substring_with_char_range(text.to_string(), &(current_ix..matched_entity.range.start));
        let entity_text = placeholder_fn(&*matched_entity.entity_name);
        processed_text = format!("{}{}{}", processed_text, prefix_text, entity_text);
        offset += entity_text.chars().count() as i16 - matched_entity.range.clone().count() as i16;
        let range_end = (matched_entity.range.end as i16 + offset) as usize;
        let new_range = range_start..range_end;
        current_ix = matched_entity.range.end;
        range_mapping.insert(new_range, matched_entity.range);
    }

    processed_text = format!(
        "{}{}",
        processed_text,
        suffix_from_char_index(text.to_string(), current_ix)
    );
    (range_mapping, processed_text)
}

fn deduplicate_overlapping_entities(entities: Vec<MatchedEntity>) -> Vec<MatchedEntity> {
    let entities_overlap = |lhs_entity: &MatchedEntity, rhs_entity: &MatchedEntity| {
        ranges_overlap(&lhs_entity.range, &rhs_entity.range)
    };
    let entity_sort_key = |entity: &MatchedEntity| -(entity.range.clone().count() as i32);
    let mut deduped = deduplicate_overlapping_items(entities, entities_overlap, entity_sort_key);
    deduped.sort_by_key(|entity| entity.range.start);
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use snips_nlu_utils::range::ranges_overlap;
    use std::ops::Range;

    #[test]
    fn test_deduplicate_items_works() {
        // Given
        let items = vec![0..3, 4..8, 0..8, 9..13];

        fn sort_key(rng: &Range<usize>) -> i32 {
            -(rng.clone().count() as i32)
        }

        // When
        let mut dedup_items = deduplicate_overlapping_items(items, ranges_overlap, sort_key);
        dedup_items.sort_by_key(|item| item.start);

        // Then
        let expected_items = vec![0..8, 9..13];
        assert_eq!(expected_items, dedup_items);
    }
}
