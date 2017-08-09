use std::f32;
use std::path;
use std::env;

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
