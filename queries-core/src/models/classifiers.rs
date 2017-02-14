use ndarray::prelude::*;

pub trait Classifier {
    fn new(weights: Array2<f64>) -> Self;
    fn run(&self, features: &Array2<f64>) -> Array2<f64>;
}

pub struct LogisticRegression {
    weights: Array2<f64>,
}

fn sigmoid(d: f64) -> f64 {
    1.0 / (1.0 + (-d).exp())
}

impl Classifier for LogisticRegression {
    fn new(weights: Array2<f64>) -> LogisticRegression {
        LogisticRegression { weights: weights }
    }

    fn run(&self, features: &Array2<f64>) -> Array2<f64> {
        let intercept = Array::from_elem((1, features.dim().1), 1.0);
        let intercepted_features = stack![Axis(0), intercept, *features];
        let mut result = self.weights.dot(&intercepted_features);
        result.mapv_inplace(sigmoid);
        result
    }
}

pub struct MulticlassLogisticRegression {
    weights: Array2<f64>,
}

impl Classifier for MulticlassLogisticRegression {
    fn new(weights: Array2<f64>) -> MulticlassLogisticRegression {
        MulticlassLogisticRegression { weights: weights }
    }

    fn run(&self, features: &Array2<f64>) -> Array2<f64> {
        let intercept = Array::from_elem((1, features.dim().1), 1.0);
        let intercepted_features = stack![Axis(0), intercept, *features];
        let mut result = self.weights.dot(&intercepted_features);
        result.mapv_inplace(|e| e.exp());
        let divider = result.map_axis(Axis(0), |v| v.scalar_sum());
        result /= &divider;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Classifier;
    use super::LogisticRegression;
    use super::MulticlassLogisticRegression;
    use testutils::assert_epsilon_eq;
    use testutils::parse_json;
    use testutils::create_array;
    use testutils::create_transposed_array;

    #[derive(Deserialize)]
    struct TestDescription {
        //description: String,
        input: InputDescription,
        output: Vec<Vec<f64>>,
    }

    #[derive(Deserialize)]
    struct InputDescription {
        weights: Vec<Vec<f64>>,
        features: Vec<Vec<f64>>,
    }

    #[test]
    fn logisitic_regression_works() {
        do_test::<LogisticRegression>("snips-sdk-tests/models/logistic_regression_predictions.json");
    }

    #[test]
    fn multiclass_logisitic_regression_works() {
        do_test::<MulticlassLogisticRegression>("snips-sdk-tests/models/multi_class_logistic_regression_predictions.json");
    }

    fn do_test<T: Classifier>(file_name: &str) {
        let descs: Vec<TestDescription> = parse_json(file_name);
        assert!(descs.len() != 0);
        for desc in descs {
            let w = create_array(&desc.input.weights);
            let f = create_transposed_array(&desc.input.features);
            let reg = T::new(w);
            let result = reg.run(&f);
            let expected_result = create_transposed_array(&desc.output);
            assert_epsilon_eq(expected_result, result, 1e-9);
        }
    }
}
