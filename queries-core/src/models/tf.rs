use std::{ fs, path, sync };
use std::io::Read;

use errors::*;
use ndarray::prelude::*;

use tensorflow::Graph;
use tensorflow::ImportGraphDefOptions;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::StepWithGraph;
use tensorflow::Tensor;

use pipeline::Probability;


pub struct TensorFlowClassifier(sync::Mutex<(Session, Graph)>);
pub struct TensorFlowCRFClassifier(TensorFlowClassifier);

pub trait Classifier {
    fn predict_proba(&self, features: &Array2<f32>) -> Result<Array2<f32>>;
    fn predict(&self, features: &Array2<f32>) -> Result<Array1<u32>>;
}

fn array_to_tensor(array: &Array2<f32>) -> Result<Tensor<f32>> {
    let shape = array.shape();
    let mut tensor: Tensor<f32> = Tensor::new(&[array.shape()[0] as u64, array.shape()[1] as u64]);
    for (&mut v1, &v2) in tensor.iter_mut().zip(array.view().iter()) {
        v1 = v2
    }
    Ok(tensor)
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
            Ok(Tensor::<f32>::new(&[2 as u64, 2 as u64]))
        };
        Ok(Array2::<f32>::zeros((1, 1)))
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use models::tf::{ TensorFlowClassifier };

    #[test]
    fn tf_classifier_works() {
        let model_path = Path::new("../data/snips-sdk-models-protobuf/models/tokens_classification/BookRestaurant_model.pb");
        let model = TensorFlowClassifier::new(model_path).unwrap();
    }
}