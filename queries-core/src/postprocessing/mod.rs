use ndarray::prelude::*;

pub fn bilou(predictions: Array1<usize>) -> Array1<usize> {
    predictions.mapv_into(|p| (p + 3) / 4)
}
