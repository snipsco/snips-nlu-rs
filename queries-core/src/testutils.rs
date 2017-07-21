#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path;
#[cfg(test)]
use std::env;

#[cfg(test)]
use ndarray::prelude::*;
#[cfg(test)]
use serde;
#[cfg(test)]
use serde_json;

#[cfg(test)]
pub fn assert_epsilon_eq_array1(a: &Array1<f32>, b: &Array1<f32>, epsilon: f32) {
    assert_eq!(a.dim(), b.dim());
    for (index, elem_a) in a.indexed_iter() {
        assert!(epsilon_eq(*elem_a, b[index], epsilon))
    }
}

#[cfg(test)]
pub fn epsilon_eq(a: f32, b: f32, epsilon: f32) -> bool {
    let diff = a - b;
    diff < epsilon && diff > -epsilon
}

#[cfg(test)]
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

#[cfg(test)]
pub fn parse_json<'a, T: for<'de> serde::Deserialize<'de>>(file_name: &str) -> T {
    let f = fs::File::open(file_path(file_name))
        .map_err(|_| format!("could not open {:?}", file_name))
        .unwrap();
    serde_json::from_reader(f)
        .map_err(|err| format!("could not parse json in {:?}\n{:?}", file_name, err))
        .unwrap()
}
