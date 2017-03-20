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

pub struct TensorFlowClassifier {
    state: sync::Mutex<(Session, Graph)>,
}
pub struct TensorFlowCRFClassifier {
    state: sync::Mutex<(Session, Graph)>,
    transition_matrix: Array2<f32>,
    num_classes: u32,
}

pub trait Classifier {
    fn predict_proba(&self, features: &Array2<f32>) -> Result<Array2<f32>>;
    fn predict(&self, features: &Array2<f32>) -> Result<Array1<usize>>;
    fn state(&self) -> &sync::Mutex<(Session, Graph)>;
    fn run(&self, features: &Array2<f32>) -> Result<Array2<f32>> {
        let x = array_to_tensor(features)?;
        let result: Result<Tensor<f32>> = {
            let mut step = StepWithGraph::new();
            let mut locked = self.state().lock().map_err(|_| "Can not take lock on TensorFlow. Mutex poisoned")?;
            let (ref mut session, ref graph) = *locked;
            step.add_input(&graph.operation_by_name_required("input")?, 0, &x);
            let res = step.request_output(&graph.operation_by_name_required("logits")?, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };

        tensor_to_array(&(result?))
    }
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

fn viterbi_decode(unary_potentials: &Array2<f32>, transition_matrix: &Array2<f32>) -> Result<Array1<usize>> {
    let num_samples = unary_potentials.rows();
    let num_classes = unary_potentials.cols();

    let mut treillis: Array1<f32> = Array::from_shape_fn(num_classes, |x| unary_potentials[(0, x)]);
    let mut viterbi: Array1<usize> = Array::zeros(num_samples);
    let mut traceback: Array2<usize> = Array::zeros((num_samples, num_classes));

    // Create the treillis
    for mut subview in traceback.outer_iter_mut() {
        let treillis_copy = treillis.to_owned();
        for (i, col) in transition_matrix.axis_iter(Axis(1)).enumerate() {
            let mut index = 0;
            let mut max_value = f32::NEG_INFINITY;
            for (j, &transition) in col.iter().enumerate() {
                let value = treillis_copy[i] + transition;
                if value > max_value {
                    index = j;
                    max_value = value;
                }
            }
            treillis[i] = max_value;
            subview[i] = index;
        }
    }

    viterbi[num_samples - 1] = {
        let mut index = 0;
        let mut max_value = f32::NEG_INFINITY;
        for (j, &value) in treillis.iter().enumerate() {
            if value > max_value {
                index = j;
                max_value = value;
            }
        }
        index
    };
    for t in 1..num_samples {
        let index: usize = num_samples - t;
        viterbi[index - 1] = traceback[(index, viterbi[index])];
    }

    Ok(viterbi)
}

impl TensorFlowClassifier {
    pub fn new<P>(model_path: P) -> Result<TensorFlowClassifier>
        where P: AsRef<path::Path>
    {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        fs::File::open(model_path)?.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let session = Session::new(&SessionOptions::new(), &graph)?;

        Ok(TensorFlowClassifier {state: sync::Mutex::new((session, graph))})
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

    fn state(&self) -> &sync::Mutex<(Session, Graph)> {
        &self.state
    }
}

impl TensorFlowCRFClassifier {
    pub fn new<P>(model_path: P, num_classes: u32) -> Result<TensorFlowCRFClassifier>
        where P: AsRef<path::Path>
    {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        fs::File::open(model_path)?.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let mut session = Session::new(&SessionOptions::new(), &graph)?;

        let result: Result<Tensor<f32>> = {
            let mut step = StepWithGraph::new();
            let res = step.request_output(&graph.operation_by_name_required("transitions")?, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };
        let transition_matrix = tensor_to_array(&(result?));

        Ok(TensorFlowCRFClassifier {
            state: sync::Mutex::new((session, graph)),
            transition_matrix: transition_matrix?,
            num_classes: num_classes
        })
    }
}

impl Classifier for TensorFlowCRFClassifier {
    fn predict_proba(&self, features: &Array2<f32>) -> Result<Array2<f32>> {
        // TODO: Replace this warning by a proper logger
        println!("Predictions for models based on a Conditional Random \
            Fields (CRF) are given by the Viterbi algorithm, which returns \
            a single most probable sequence");
        let predictions = self.predict(&features)?;
        let mut probas = Array2::<f32>::zeros((features.rows(), self.num_classes as usize));
        for (i, &j) in predictions.view().iter().enumerate() {
            probas[(i, j)] = 1.;
        }

        Ok(probas)
    }

    fn predict(&self, features: &Array2<f32>) -> Result<Array1<usize>> {
        let unary_potentials = self.run(&features)?;
        let predictions = viterbi_decode(&unary_potentials, &self.transition_matrix);

        predictions
    }

    fn state(&self) -> &sync::Mutex<(Session, Graph)> {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use ndarray::prelude::*;

    use models::tf::{TensorFlowClassifier, TensorFlowCRFClassifier, Classifier};

    #[test]
    fn tf_classifier_works() {
        let model_path = Path::new("../data/snips-sdk-models-protobuf/models/tokens_classification/BookRestaurant_model.pb");
        let features = Array2::<f32>::zeros((11, 1881));

        // CNN
        let model = TensorFlowClassifier::new(model_path).unwrap();
        let probas = model.predict_proba(&features);
        let predictions = model.predict(&features);

        println!("probas: {:?}", probas);
        println!("predictions: {:?}", predictions);

        // CNN + CRF
        let model_crf = TensorFlowCRFClassifier::new(model_path, 4).unwrap();
        let probas = model_crf.predict_proba(&features);
        let predictions = model_crf.predict(&features);

        println!("probas (CRF): {:?}", probas);
        println!("predictions (CRF): {:?}", predictions);
    }
}