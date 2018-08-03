pub type IntentName = String;
pub type SlotName = String;
pub type EntityName = String;

#[cfg(test)]
pub fn file_path(filename: &str) -> ::std::path::PathBuf {
    ::dinghy_test::try_test_file_path("data")
        .unwrap_or_else(|| "../data".into())
        .join(filename)
}

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

pub fn deduplicate_overlapping_items<I, O, R>(
    items: Vec<I>,
    overlap: O,
    resolve: R
) -> Vec<I>
    where I: Clone, O: Fn(&I, &I) -> bool, R: Fn(I, I) -> I
{
    let mut deduped: Vec<I> = Vec::with_capacity(items.len());
    for item in items {
        let conflicting_item_index = deduped
            .iter()
            .position(|existing_item| overlap(&item, &existing_item));

        if let Some(index) = conflicting_item_index {
            let resolved_item = resolve(deduped[index].clone(), item);
            deduped[index] = resolved_item;
        } else {
            deduped.push(item);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::repeat_n;
    use std::collections::HashSet;

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
            "hello".to_string(),
            "blue bird".to_string(),
            "blue".to_string(),
            "hello world".to_string(),
            "blue bird".to_string()
        ];

        fn overlap(lhs_str: &String, rhs_str: &String) -> bool {
            lhs_str.starts_with(rhs_str) || rhs_str.starts_with(lhs_str)
        }

        fn resolve(lhs_str: String, rhs_str: String) -> String {
            if lhs_str.len() > rhs_str.len() {
                lhs_str
            } else {
                rhs_str
            }
        }

        // When
        let dedup_items = deduplicate_overlapping_items(items, overlap, resolve);

        // Then
        let expected_items = vec!["hello world".to_string(), "blue bird".to_string()];
        assert_eq!(expected_items, dedup_items);
    }
}
