use std::io::prelude::*;
use std::fs::File;

use serde_json::from_str;
use serde::de::Deserialize;
use ndarray::prelude::*;
use file_path;

pub fn parse_json<T: Deserialize>(file_name: &str) -> T {
    let mut f = File::open(file_path(file_name)).unwrap();
    let mut s = String::new();
    assert!(f.read_to_string(&mut s).is_ok());
    from_str::<T>(&s).unwrap()
}

pub fn assert_epsilon_eq(a: Array2<f32>, b: Array2<f32>, epsilon: f32) {
    for (index, elem_a) in a.indexed_iter() {
        assert!(epsilon_eq(*elem_a, b[index], epsilon))
    }
}

pub fn epsilon_eq(a: f32, b: f32, epsilon: f32) -> bool {
    let diff = a - b;
    diff < epsilon && diff > -epsilon
}

pub fn create_array(input: &Vec<Vec<f32>>) -> Array2<f32> {
    Array::from_shape_fn((input.len(), input[0].len()), |x| input[x.0][x.1])
}

pub fn create_transposed_array(input: &Vec<Vec<f32>>) -> Array2<f32> {
    Array::from_shape_fn((input[0].len(), input.len()), |x| input[x.1][x.0])
}
