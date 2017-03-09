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

pub trait CNN {
    fn run(&self, features: &Array2<f32>) -> Result<Array2<f32>>;
}

pub struct TensorflowCNN(sync::Mutex<(Session,Graph)>);

unsafe impl Send for TensorflowCNN {}
unsafe impl Sync for TensorflowCNN {}

impl TensorflowCNN {
    pub fn new<P: AsRef<path::Path>>(model_path: P) -> Result<TensorflowCNN> {
        let mut graph = Graph::new();
        let mut proto = Vec::new();
        fs::File::open(model_path)?.read_to_end(&mut proto)?;

        graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
        let session = Session::new(&SessionOptions::new(), &graph)?;

        Ok(TensorflowCNN(sync::Mutex::new((session, graph))))
    }
}

impl CNN for TensorflowCNN {
    fn run(&self, features: &Array2<f32>) -> Result<Array2<Probability>> {
        let transposed_array = features.t();
        let tokens_count = transposed_array.shape()[0];
        let features_count = transposed_array.shape()[1];

        let mut x: Tensor<f32> = Tensor::new(&[tokens_count as u64, features_count as u64]);
        for row in 0..tokens_count {
            for col in 0..features_count {
                x[row * features_count + col] = *transposed_array.get((row, col)).unwrap(); // TODO: Geometry is checked ?
            }
        }

        let mut step = StepWithGraph::new();
        let tensor_res: Result<Tensor<f32>> = {
            let mut locked = self.0.lock().map_err(|_| "Can not take lock on Tensorflow. Mutex poisoned.")?;
            let (ref mut session, ref graph) = *locked;
            step.add_input(&graph.operation_by_name_required("input")?, 0, &x);
            let res = step.request_output(&graph.operation_by_name_required("logits")?, 0);

            session.run(&mut step)?;

            Ok(step.take_output(res)?)
        };
        let tensor_res = tensor_res?;

        let mut vec = Vec::with_capacity(tensor_res.data().len());
        vec.extend_from_slice(&tensor_res.data());

        let res_shape = (tensor_res.dims()[0] as usize, tensor_res.dims()[1] as usize);
        Ok(Array::from_vec(vec).into_shape(res_shape)?)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use ndarray::prelude::*;

    use models::cnn::{ CNN, TensorflowCNN };

    #[test]
    #[ignore]
    fn cnn_works() {
        let model_path = Path::new("../data/snips-sdk-models-protobuf/tokens_classification/Cnn_BookRestaurant_bookRestaurant.pb");
        let cnn = TensorflowCNN::new(model_path).unwrap();
        let features = arr2(&[[1.0], [2.0]]);

        let probabilities = cnn.run(&features);

        println!("probabilities: {:?}", probabilities);
    }
}
