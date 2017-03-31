use ndarray::prelude::*;

pub fn bilou(predictions: Array1<usize>) -> Array1<usize> {
    predictions
        .into_iter()
        .map(|p| (p + 3) / 4)
        .collect()
}
