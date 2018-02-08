use std::env;
use std::path;

pub fn file_path(file_name: &str) -> path::PathBuf {
    if cfg!(any(target_os = "ios", target_os = "android")) || env::var("DINGHY").is_ok() {
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_data/data")
            .join(file_name)
    } else {
        path::PathBuf::from("../data").join(file_name)
    }
}

pub fn permutations<T: Copy>(v: &[T], permutation_length: i32) -> Vec<Vec<T>> {
    if permutation_length > v.len() as i32 {
        panic!("permutation_length must be greater than 0 and less than the length of v")
    };

    if permutation_length == 0 {
        return vec![vec![]];
    }

    let mut perms: Vec<Vec<T>> = vec![];
    for (i, tail) in v.iter().enumerate() {
        let sub_vec = exclude_from_list(v, i);
        for mut p in permutations(&*sub_vec, permutation_length - 1).into_iter() {
            p.push(tail.clone());
            perms.push(p);
        }
    }
    perms
}


fn partial_cartesian<'b, T>(acc: Vec<Vec<&'b T>>, a: &'b [T]) -> Vec<Vec<&'b T>> {
    acc.into_iter().flat_map(|xs| {
        a.iter().map(|y| {
            let mut vec = xs.clone();
            vec.push(&y);
            vec
        }).collect::<Vec<_>>()
    }).collect()
}


pub fn product<'a, T>(v: &'a [&'a [T]]) -> Vec<Vec<&'a T>> {
    v.split_first()
        .map_or(vec![], |(head, tail)| {
            let init: Vec<Vec<&T>> = head.iter().map(|n| vec![n]).collect();
            tail.iter().cloned().fold(init, |vec, list| {
                partial_cartesian(vec, list)
            })
        })
}


fn exclude_from_list<T: Copy>(list: &[T], i: usize) -> Vec<T> {
    list.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, a)| *a).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use itertools::repeat_n;

    #[test]
    fn permutations_works() {
        // Given
        let my_vec = vec![1, 2, 3];

        // When
        let perms = permutations(&*my_vec, 3);

        // Then
        let expected_perms = hashset![
            vec![1, 2, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![1, 3, 2],
            vec![3, 2, 1],
            vec![2, 1, 3],
        ];

        assert_eq!(expected_perms.len(), perms.len());
        assert_eq!(expected_perms, HashSet::from_iter(perms))
    }


    #[test]
    fn product_works() {
        // Given
        let pool: Vec<Vec<i32>> = repeat_n(0..2, 3)
            .map(|range| range.into_iter().collect())
            .collect();

        let ref_pool: Vec<&[i32]> = pool
            .iter()
            .map(|v| &v[..])
            .collect();

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
