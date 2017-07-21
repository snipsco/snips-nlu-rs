use std::ops;
use std::f32;

use ndarray::prelude::*;


pub fn ranges_overlap(r1: &ops::Range<usize>, r2: &ops::Range<usize>) -> bool {
    r1.start < r2.end && r1.end > r2.start
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_overlap_works() {
        let test_cases = vec![
            (3..6, 4..7, true),
            (3..6, 4..5, true),
            (3..6, 7..9, false),
            (3..6, 6..7, false),
        ];

        for (r1, r2, expected_result) in test_cases {
            assert_eq!(ranges_overlap(&r1, &r2), expected_result);
            assert_eq!(ranges_overlap(&r2, &r1), expected_result);
        }
    }
}
