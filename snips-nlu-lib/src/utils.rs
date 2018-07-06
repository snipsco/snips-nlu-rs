use std::path::Path;

use errors::*;

pub trait FromPath {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized;
}

pub fn file_path(filename: &str) -> ::std::path::PathBuf {
    ::dinghy_test::try_test_file_path("data")
        .unwrap_or("../data".into())
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
}
