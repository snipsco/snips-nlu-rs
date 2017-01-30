use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json;
use rustc_serialize::Decodable;
use ndarray::prelude::*;

pub fn parse_json<T: Decodable>(file_name: &str) -> T {
    let mut f = File::open(file_name).unwrap();
    let mut s = String::new();
    assert!(f.read_to_string(&mut s).is_ok());
    json::decode::<T>(&s).unwrap()
}

pub fn assert_epsilon_eq(a: Array2<f64>, b: Array2<f64>, epsilon: f64) {
    for (index, elem_a) in a.indexed_iter() {
        assert!(epsilon_eq(*elem_a, b[index], epsilon))
    }
}

pub fn epsilon_eq(a: f64, b: f64, epsilon: f64) -> bool {
    let diff = a - b;
    diff < epsilon && diff > -epsilon
}
