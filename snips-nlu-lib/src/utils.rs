use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use zip::ZipArchive;

use errors::*;
use failure::ResultExt;

pub type IntentName = String;
pub type SlotName = String;
pub type EntityName = String;

fn partial_cartesian<'b, T>(acc: Vec<Vec<&'b T>>, a: &'b [T]) -> Vec<Vec<&'b T>> {
    acc.into_iter()
        .flat_map(|xs| {
            a.iter()
                .map(|y| {
                    let mut vec = xs.clone();
                    vec.push(y);
                    vec
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn product<'a, T>(v: &'a [&'a [T]]) -> Vec<Vec<&'a T>> {
    v.split_first().map_or(vec![], |(head, tail)| {
        let init: Vec<Vec<&T>> = head.iter().map(|n| vec![n]).collect();
        tail.iter()
            .cloned()
            .fold(init, |vec, list| partial_cartesian(vec, list))
    })
}

pub fn deduplicate_overlapping_items<I, O, S, K>(
    items: Vec<I>,
    overlap: O,
    sort_key_fn: S
) -> Vec<I>
    where I: Clone, O: Fn(&I, &I) -> bool, S: FnMut(&I) -> K, K: Ord
{
    let mut sorted_items = items.clone();
    sorted_items.sort_by_key(sort_key_fn);
    let mut deduplicated_items: Vec<I> = Vec::with_capacity(items.len());
    for item in sorted_items {
        if !deduplicated_items.iter().any(|dedup_item| overlap(dedup_item, &item)) {
            deduplicated_items.push(item);
        }
    }
    deduplicated_items
}

pub fn extract_nlu_engine_zip_archive<R: io::Read + io::Seek>(
    zip_reader: R,
    dest_path: &Path
) -> Result<PathBuf> {
    let mut archive = ZipArchive::new(zip_reader)
        .with_context(|_| "Could not read nlu engine zip data")?;
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
    let first_archive_file = archive
        .by_index(0)?
        .sanitized_name();
    let engine_dir_path = first_archive_file
        .components()
        .find(|component| if let Component::Normal(_) = component { true } else { false })
        .ok_or_else(|| format_err!("Trained engine archive is incorrect"))?
        .as_os_str();
    let engine_dir_name = engine_dir_path
        .to_str()
        .ok_or_else(|| format_err!("Engine directory name is empty"))?;
    Ok(dest_path.join(engine_dir_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::repeat_n;
    use std::collections::HashSet;
    use std::ops::Range;
    use nlu_utils::range::ranges_overlap;

    #[test]
    fn product_works() {
        // Given
        let pool: Vec<Vec<i32>> = repeat_n(0..2, 3)
            .map(|range| range.into_iter().collect())
            .collect();

        let ref_pool: Vec<&[i32]> = pool.iter().map(|v| &v[..]).collect();

        // When
        let prod: Vec<Vec<&i32>> = product(&ref_pool[..]);

        // Then
        let expected_output: HashSet<Vec<&i32>> = hashset!(
            vec![&0, &0, &0],
            vec![&0, &0, &1],
            vec![&0, &1, &0],
            vec![&0, &1, &1],
            vec![&1, &0, &0],
            vec![&1, &0, &1],
            vec![&1, &1, &0],
            vec![&1, &1, &1],
        );

        assert_eq!(expected_output.len(), prod.len());
        for p in prod {
            assert!(expected_output.contains(&p));
        }
    }

    #[test]
    fn test_deduplicate_items_works() {
        // Given
        let items = vec![
            0..3,
            4..8,
            0..8,
            9..13
        ];

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
