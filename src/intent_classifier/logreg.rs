use ndarray::prelude::*;
use ndarray::{array, stack};

use crate::errors::*;

/// The multiclass probability estimates are derived from binary (one-vs.-rest)
/// estimates by simple normalization
pub struct MulticlassLogisticRegression {
    /// matrix with shape (f, c)
    /// ------------------------
    ///
    /// - f = number of features
    /// - c = number of classes
    weights: Array2<f32>,
}

impl MulticlassLogisticRegression {
    fn nb_features(&self) -> usize {
        // without intercept
        self.weights.dim().0 - 1
    }

    fn nb_classes(&self) -> usize {
        self.weights.dim().1
    }

    fn is_binary(&self) -> bool {
        self.nb_classes() == 1
    }
}

impl MulticlassLogisticRegression {
    pub fn new(intercept: Array1<f32>, weights: Array2<f32>) -> Result<Self> {
        let nb_classes = intercept.dim();
        let reshaped_intercept = intercept.into_shape((1, nb_classes))?;
        let weights_with_intercept = stack![Axis(0), reshaped_intercept, weights];
        Ok(Self {
            weights: weights_with_intercept,
        })
    }

    pub fn run(
        &self,
        features: &ArrayView1<f32>,
        filtered_out_indexes: Option<Vec<usize>>,
    ) -> Result<Array1<f32>> {
        let reshaped_features = features.into_shape((1, self.nb_features()))?;
        let reshaped_features = stack![Axis(1), array![[1.]], reshaped_features];
        let mut result = reshaped_features
            .dot(&self.weights)
            .into_shape(self.nb_classes())?;
        result.mapv_inplace(logit);
        if self.is_binary() {
            return Ok(arr1(&[1.0 - result[0], result[0]]));
        }
        if let Some(indexes) = filtered_out_indexes {
            if !indexes.is_empty() {
                for index in indexes {
                    result[index] = 0.0;
                }
                let divider = result.scalar_sum();
                result /= divider;
            }
        }
        Ok(result)
    }
}

fn logit(x: f32) -> f32 {
    1. / (1. + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::MulticlassLogisticRegression;
    use crate::testutils::assert_epsilon_eq_array1;
    use ndarray::array;

    #[test]
    fn multiclass_logistic_regression_works() {
        // Given
        let intercept = array![0.98, 0.32, -0.76];
        let weights = array![
            [2.5, -0.6, 0.5],
            [1.2, 1.2, -2.7],
            [1.5, 0.1, -3.2],
            [-0.9, 1.4, 1.8]
        ];

        let features = array![0.4, -2.3, 1.9, 1.3];
        let regression = MulticlassLogisticRegression::new(intercept, weights).unwrap();

        // When
        let predictions = regression.run(&features.view(), None).unwrap();

        // Then
        let expected_predictions = array![0.7109495, 0.3384968, 0.8710191];
        assert_epsilon_eq_array1(&predictions, &expected_predictions, 1e-06);
    }

    #[test]
    fn multiclass_logistic_regression_works_when_binary() {
        // Given
        let intercept = array![0.98];
        let weights = array![[2.5], [1.2], [1.5], [-0.9]];

        let features = array![0.4, -2.3, 1.9, 1.3];
        let regression = MulticlassLogisticRegression::new(intercept, weights).unwrap();

        // When
        let predictions = regression.run(&features.view(), None).unwrap();

        // Then
        let expected_predictions = array![0.2890504, 0.7109495];
        assert_epsilon_eq_array1(&predictions, &expected_predictions, 1e-06);
    }

    #[test]
    fn multiclass_logistic_regression_works_with_filtered_out_indexes() {
        // Given
        let intercept = array![0.98, 0.32, -0.76];
        let weights = array![
            [2.5, -0.6, 0.5],
            [1.2, 1.2, -2.7],
            [1.5, 0.1, -3.2],
            [-0.9, 1.4, 1.8]
        ];

        let features = array![0.4, -2.3, 1.9, 1.3];

        let filtered_out_indexes = Some(vec![2]);
        let regression = MulticlassLogisticRegression::new(intercept, weights).unwrap();

        // When
        let predictions = regression
            .run(&features.view(), filtered_out_indexes)
            .unwrap();

        // Then
        let expected_predictions = array![0.67745198, 0.32254802, 0.0];
        assert_epsilon_eq_array1(&predictions, &expected_predictions, 1e-06);
    }
}
