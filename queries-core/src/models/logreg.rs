use ndarray::prelude::*;
use errors::*;

pub struct MulticlassLogisticRegression {
    // A matrix with shape (f, c)
    // f = number of features
    // c = number of classes
    weights: Array2<f32>,
}

impl MulticlassLogisticRegression {
    pub fn new(intercept: Array1<f32>, weights: Array2<f32>) -> Result<Self> {
        let nb_classes = intercept.dim();
        let reshaped_intercept = intercept.into_shape((1, nb_classes))?;
        let weights_with_intercept = stack![Axis(0), reshaped_intercept, weights];
        Ok(Self { weights: weights_with_intercept })
    }

    pub fn run(&self, features: &ArrayView1<f32>) -> Result<Array1<f32>> {
        let nb_features = features.dim();
        let (_, nb_classes) = self.weights.dim();

        let reshaped_features = features.into_shape((1, nb_features))?;
        let reshaped_features = stack![Axis(1), array![[1.]], reshaped_features];
        let mut result = reshaped_features.dot(&self.weights).into_shape(nb_classes)?;
        result.mapv_inplace(|e| e.exp());
        let divider = result.scalar_sum();
        result /= divider;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::MulticlassLogisticRegression;
    use testutils::assert_epsilon_eq_array1;

    #[test]
    fn multiclass_logistic_regression_works() {
        // Given
        let intercept = array![0.98, 0.32, -0.76];
        let weights = array![[ 2.5, -0.6,  0.5],
                             [ 1.2,  2.2, -2.7],
                             [ 1.5,  0.1, -3.2],
                             [-0.9, -2.4,  1.8]];

        let features = array![0.4, -2.3, 1.9, 1.3];
        let regression = MulticlassLogisticRegression::new(intercept, weights).unwrap();

        // When
        let predictions = regression.run(&features.view()).unwrap();

        // Then
        let expected_predictions = array![2.66969214e-01, 3.98406851e-05, 7.32990945e-01];
        assert_epsilon_eq_array1(&predictions, &expected_predictions, 1e-06);
    }
}
