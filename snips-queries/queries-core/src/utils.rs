use std::env;
use std::f32;
use std::path;

use ndarray::prelude::*;

pub fn argmax(arr: &Array1<f32>) -> (usize, f32) {
    let mut index = 0;
    let mut max_value = f32::NEG_INFINITY;
    for (j, &value) in arr.iter().enumerate() {
        if value > max_value {
            index = j;
            max_value = value;
        }
    }
    (index, max_value)
}

pub fn file_path(file_name: &str) -> path::PathBuf {
    if env::var("DINGHY").is_ok() {
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

fn exclude_from_list<T: Copy>(list: &[T], i: usize) -> Vec<T> {
    list.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, a)| *a).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;

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
}
