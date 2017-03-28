use std::sync;
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
    input_node_name: String,
    output_node_name: String,
}

/// We need the classifiers to be both [`Send`] and [`Sync`] in order to be able to use them in a
/// multi-threaded environment. We should ensure that the mutex is enough for that (the rust type
/// system seems not to be confident about that because there's a raw pointer to the TF graph,
/// hence the unsafe impl)
unsafe impl Send for TensorFlowClassifier {}
unsafe impl Sync for TensorFlowClassifier {}

pub struct TensorFlowCRFClassifier {
    state: sync::Mutex<(Session, Graph)>,
    transition_matrix: Array2<f32>,
    num_classes: u32,
    input_node_name: String,
    output_node_name: String,
}

/// We need the classifiers to be both [`Send`] and [`Sync`] in order to be able to use them in a
/// multi-threaded environment. We should ensure that the mutex is enough for that (the rust type
/// system seems not to be confident about that because there's a raw pointer to the TF graph,
/// hence the unsafe impl)
unsafe impl Send for TensorFlowCRFClassifier {}
unsafe impl Sync for TensorFlowCRFClassifier {}

pub trait Classifier {
    fn predict_proba(&self, features: &ArrayView2<f32>) -> Result<Array2<f32>>;
    fn predict(&self, features: &ArrayView2<f32>) -> Result<Array1<usize>>;
    fn state(&self) -> &sync::Mutex<(Session, Graph)>;
    fn input_node(&self) -> &str;
    fn output_node(&self) -> &str;
    fn run(&self, features: &ArrayView2<f32>) -> Result<Array2<f32>> {
        let x = array_to_tensor(features)?;
        let result: Result<Tensor<f32>> = {
            let mut step = StepWithGraph::new();
            let mut locked =
                self.state().lock().map_err(|_| "Can not take lock on TensorFlow. Mutex poisoned")?;
            let (ref mut session, ref graph) = *locked;
            step.add_input(&graph.operation_by_name_required(self.input_node())?, 0, &x);
            let res =
                step.request_output(&graph.operation_by_name_required(self.output_node())?, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };

        tensor_to_array(&(result?))
    }
}

fn array_to_tensor(array: &ArrayView2<f32>) -> Result<Tensor<f32>> {
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

fn viterbi_decode(unary_potentials: &Array2<f32>,
                  transition_matrix: &Array2<f32>)
                  -> Result<Array1<usize>> {
    let num_samples = unary_potentials.rows();
    let num_classes = unary_potentials.cols();

    let mut treillis: Array1<f32> = Array::from_shape_fn(num_classes, |x| unary_potentials[(0, x)]);
    let mut viterbi: Array1<usize> = Array::zeros(num_samples);
    let mut traceback: Array2<usize> = Array::zeros((num_samples - 1, num_classes));

    // Create the treillis
    for (t, mut subview) in traceback.outer_iter_mut().enumerate() {
        let treillis_copy = treillis.to_owned();
        for (i, col) in transition_matrix.axis_iter(Axis(1)).enumerate() {
            let mut index = 0;
            let mut max_value = f32::NEG_INFINITY;
            for (j, &transition) in col.iter().enumerate() {
                let value = treillis_copy[j] + transition;
                if value > max_value {
                    index = j;
                    max_value = value;
                }
            }
            treillis[i] = unary_potentials[(t + 1, i)] + max_value;
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
        viterbi[index - 1] = traceback[(index - 1, viterbi[index])];
    }

    Ok(viterbi)
}

impl TensorFlowClassifier {
    pub fn new(model: &mut Read,
               input_node_name: String,
               output_node_name: String)
               -> Result<TensorFlowClassifier> {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        model.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let session = Session::new(&SessionOptions::new(), &graph)?;

        Ok(TensorFlowClassifier {
            state: sync::Mutex::new((session, graph)),
            input_node_name: input_node_name,
            output_node_name: output_node_name,
        })
    }
}

impl Classifier for TensorFlowClassifier {
    fn predict_proba(&self, features: &ArrayView2<f32>) -> Result<Array2<f32>> {
        let mut logits = self.run(features)?;
        let num_classes = logits.cols();
        if num_classes > 1 {
            for mut row in logits.outer_iter_mut() {
                let max = *(row.iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Less))
                    .unwrap());
                for value in row.iter_mut() {
                    *value = (*value - max).exp()
                }
                let sum_exponent: f32 = row.iter().sum();
                row /= sum_exponent;
            }
        } else {
            for mut value in logits.iter_mut() {
                *value = 1. / (1. + (-*value).exp())
            }
        }

        Ok(logits)
    }

    fn predict(&self, features: &ArrayView2<f32>) -> Result<Array1<usize>> {
        let logits = self.run(features)?;
        let num_classes = logits.cols();
        let predictions: Array1<usize> = if num_classes > 1 {
            Array::from_iter(logits.outer_iter().map(|row| {
                let mut index = 0;
                let mut max_value = f32::NEG_INFINITY;
                for (j, &value) in row.iter().enumerate() {
                    if value > max_value {
                        index = j;
                        max_value = value;
                    }
                }
                index
            }))
        } else {
            Array::from_iter(logits.iter()
                .map(|value| (*value > 0.) as usize))
        };

        Ok(predictions)
    }

    fn state(&self) -> &sync::Mutex<(Session, Graph)> {
        &self.state
    }

    fn input_node(&self) -> &str {
        &self.input_node_name
    }

    fn output_node(&self) -> &str {
        &self.output_node_name
    }
}

impl TensorFlowCRFClassifier {
    pub fn new(model: &mut Read,
               num_classes: u32,
               input_node_name: String,
               output_node_name: String,
               transition_matrix_node_name: &str)
               -> Result<TensorFlowCRFClassifier> {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        model.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let mut session = Session::new(&SessionOptions::new(), &graph)?;

        let result: Result<Tensor<f32>> = {
            let mut step = StepWithGraph::new();
            let node = &graph.operation_by_name_required(transition_matrix_node_name)?;
            let res = step.request_output(node, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };
        let transition_matrix = tensor_to_array(&(result?));

        Ok(TensorFlowCRFClassifier {
            state: sync::Mutex::new((session, graph)),
            transition_matrix: transition_matrix?,
            num_classes: num_classes,
            input_node_name: input_node_name,
            output_node_name: output_node_name,
        })
    }
}

impl Classifier for TensorFlowCRFClassifier {
    fn predict_proba(&self, features: &ArrayView2<f32>) -> Result<Array2<f32>> {
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

    fn predict(&self, features: &ArrayView2<f32>) -> Result<Array1<usize>> {
        let unary_potentials = self.run(&features)?;
        let predictions = viterbi_decode(&unary_potentials, &self.transition_matrix);

        predictions
    }

    fn state(&self) -> &sync::Mutex<(Session, Graph)> {
        &self.state
    }

    fn input_node(&self) -> &str {
        &self.input_node_name
    }

    fn output_node(&self) -> &str {
        &self.output_node_name
    }
}

#[cfg(test)]
mod test {
    use std::path;
    use std::fs::File;
    use std::cmp;

    use ndarray::prelude::*;
    use protobuf;

    use config::{AssistantConfig, IntentConfig, FileBasedAssistantConfig};
    use protos::model_configuration::ModelConfiguration;
    use models::tf::{TensorFlowClassifier, TensorFlowCRFClassifier, Classifier};
    use testutils::{epsilon_eq, assert_epsilon_eq};

    #[test]
    fn tf_classifier_predict_proba_works() {
        let model_path = path::PathBuf::from("../data/tests/models/tf/graph_logistic_regression.pb");
        let mut model_file = Box::new(File::open(model_path).unwrap());
        let model = TensorFlowClassifier::new(&mut model_file,
                                              "inputs".to_string(),
                                              "logits".to_string()).unwrap();

        // Data
        let x = arr2(&[[0.1, 0.1, 0.1],
                       [0.3, 0.5, 0.8]]);
        // TensorFlow
        let proba_tf = model.predict_proba(&x.view()).unwrap();

        // ndarray
        let w = arr2(&[[ 0.2,  0.3,  0.5,  0.7, 0.11],
                       [0.13, 0.17, 0.19, 0.23, 0.29],
                       [0.31, 0.37, 0.41, 0.43, 0.47]]);
        let mut proba_nd = x.dot(&w);
        // TODO: Have the softmax function below in an utils 
        for mut row in proba_nd.outer_iter_mut() {
            let max = *(row.iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Less))
                .unwrap());
            for value in row.iter_mut() {
                *value = (*value - max).exp()
            }
            let sum_exponent: f32 = row.iter().sum();
            row /= sum_exponent;
        }

        assert_eq!(proba_tf.shape(), &[2, 5]);
        assert_epsilon_eq(&proba_tf, &proba_nd, 1e-6);
        // for row in proba_tf.outer_iter() {
        //     assert!(epsilon_eq(row.iter().sum(), 1.0, 1e-6));
        // }
        // TODO: test if all elements of proba_tf is between 0. and 1.
    }

    // #[test]
    // fn tf_classifier_works() {
    //     let intent_config = FileBasedAssistantConfig::default()
    //         .get_intent_configuration("BookRestaurant").unwrap();
    //     let pb_intent_config = intent_config.get_pb_config().unwrap();
    //     let mut tokens_classifier_config = intent_config.get_file(path::Path::new(pb_intent_config.get_tokens_classifier_path())).unwrap();
    //     let pb_model_config = protobuf::parse_from_reader::<ModelConfiguration>(&mut tokens_classifier_config).unwrap();
    //     let model_path = pb_model_config.get_model_path();
    //     let tf_model = &mut intent_config.get_file(path::Path::new(model_path)).unwrap();

    //     // TODO: Get the number of features from the BookRestaurant_config file
    //     let features = Array2::<f32>::zeros((11, 1881));

    //     // CNN
    //     let model = TensorFlowClassifier::new(tf_model,
    //                                           pb_model_config.get_input_node().to_string(),
    //                                           pb_model_config.get_output_node().to_string()).unwrap();
    //     let probas = model.predict_proba(&features.view());
    //     let predictions = model.predict(&features.view());

    //     println!("probas: {:?}", probas);
    //     println!("predictions: {:?}", predictions);
    // }

    // #[test]
    // fn tf_classifier_crf_works() {
    //     let intent_config = FileBasedAssistantConfig::default()
    //         .get_intent_configuration("BookRestaurant").unwrap();
    //     let pb_intent_config = intent_config.get_pb_config().unwrap();
    //     let mut tokens_classifier_config = intent_config.get_file(path::Path::new(pb_intent_config.get_tokens_classifier_path())).unwrap();
    //     let pb_tokens_config = protobuf::parse_from_reader::<ModelConfiguration>(&mut tokens_classifier_config).unwrap();
    //     let model_path = pb_tokens_config.get_model_path();
    //     let tf_model = &mut intent_config.get_file(path::Path::new(model_path)).unwrap();

    //     // TODO: Get the number of features from the BookRestaurant_config file
    //     let features = Array2::<f32>::zeros((11, 1881));

    //     // CNN + CRF
    //     let model = TensorFlowCRFClassifier::new(tf_model,
    //                                              pb_intent_config.get_slots().len() as u32,
    //                                              pb_tokens_config.get_input_node().to_string(),
    //                                              pb_tokens_config.get_output_node().to_string(),
    //                                              pb_tokens_config.get_transition_matrix_node()).unwrap();
    //     let probas = model.predict_proba(&features.view());
    //     let predictions = model.predict(&features.view());

    //     println!("probas (CRF): {:?}", probas);
    //     println!("predictions (CRF): {:?}", predictions);
    // }
}
