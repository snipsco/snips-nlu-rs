use std::{ fs, path, sync };
use std::io::Read;
use std::f32;
use std::cmp;

use errors::*;
use ndarray::prelude::*;

use tensorflow::Graph;
use tensorflow::ImportGraphDefOptions;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::StepWithGraph;
use tensorflow::Tensor;

pub struct TensorFlowClassifier(sync::Mutex<(Session, Graph)>);
pub struct TensorFlowCRFClassifier(TensorFlowClassifier);

pub trait Classifier {
    fn predict_proba(&self, features: &Array2<f32>) -> Result<Array2<f32>>;
    fn predict(&self, features: &Array2<f32>) -> Result<Array1<usize>>;
}

fn array_to_tensor(array: &Array2<f32>) -> Result<Tensor<f32>> {
    let shape = array.shape();
    let mut tensor: Tensor<f32> = Tensor::new(&[shape[0] as u64, shape[1] as u64]);
    for (i, &elem) in array.view().iter().enumerate() {
        tensor[i] = elem
    }

    Ok(tensor)
}

fn tensor_to_array(tensor: &Tensor<f32>) -> Result<Array2<f32>> {
    let shape = tensor.dims();
    let mut vec = Vec::with_capacity(tensor.len());
    vec.extend_from_slice(&tensor.data());
    let array = Array::from_vec(vec).into_shape((shape[0] as usize, shape[1] as usize))?;

    Ok(array)
}

impl TensorFlowClassifier {
    pub fn new<P: AsRef<path::Path>>(model_path: P) -> Result<TensorFlowClassifier> {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        fs::File::open(model_path)?.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let session = Session::new(&SessionOptions::new(), &graph)?;

        Ok(TensorFlowClassifier(sync::Mutex::new((session, graph))))
    }

    fn run(&self, features: &Array2<f32>) -> Result<Array2<f32>> {
        let x = array_to_tensor(features)?;
        let mut step = StepWithGraph::new();
        let result: Result<Tensor<f32>> = {
            let mut locked = self.0.lock().map_err(|_| "Can not take lock on TensorFlow. Mutex poisoned")?;
            let (ref mut session, ref graph) = *locked;
            step.add_input(&graph.operation_by_name_required("input")?, 0, &x);
            let res = step.request_output(&graph.operation_by_name_required("logits")?, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };
        
        tensor_to_array(&(result?))
    }
}

impl Classifier for TensorFlowClassifier {
    fn predict_proba(&self, features: &Array2<f32>) -> Result<Array2<f32>> {
        let mut logits = self.run(features)?;
        for mut row in logits.outer_iter_mut() {
            let max = *(row.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Less)).unwrap());
            for value in row.iter_mut() {
                *value = (*value - max).exp()
            }
            let sum_exponent: f32 = row.iter().sum();
            row /= sum_exponent;
        }

        Ok(logits)
    }

    fn predict(&self, features: &Array2<f32>) -> Result<Array1<usize>> {
        let logits = self.run(features)?;
        let predictions: Array1<usize> = Array::from_iter(logits.outer_iter().map(|row| {
            let mut index = 0;
            let mut max_value = f32::NEG_INFINITY;
            for (j, &value) in row.iter().enumerate() {
                if value > max_value {
                    index = j;
                    max_value = value;
                }
            }

            index
        }));

        Ok(predictions)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use ndarray::prelude::*;

    use models::tf::{ TensorFlowClassifier, Classifier };

    #[test]
    fn tf_classifier_works() {
        let model_path = Path::new("../data/snips-sdk-models-protobuf/models/tokens_classification/BookRestaurant_model.pb");
        let model = TensorFlowClassifier::new(model_path).unwrap();
        let features = Array2::<f32>::zeros((11, 1358));
        let probas = model.predict_proba(&features);
        let predictions = model.predict(&features);

        println!("probas: {:?}", probas);
        println!("predictions: {:?}", predictions);
    }
}