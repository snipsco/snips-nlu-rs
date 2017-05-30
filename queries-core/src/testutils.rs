use ndarray::prelude::*;

pub fn assert_epsilon_eq_array1(a: &Array1<f32>, b: &Array1<f32>, epsilon: f32) {
    assert_eq!(a.dim(), b.dim());
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
