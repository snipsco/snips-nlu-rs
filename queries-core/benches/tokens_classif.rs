#[macro_use]
extern crate bencher;

extern crate snips_queries_core;

use std::{path};

use bencher::Bencher;
use snips_queries_core::models::cnn::TensorflowCNN;
use snips_queries_core::pipeline::tokens_classifier::ProtobufTokensClassifier;
use snips_queries_core::pipeline::tokens_classifier::TokensClassifier;
use snips_queries_core::preprocess;

fn load_classifier(bench: &mut Bencher) {
    let cnn_path = path::Path::new("../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb");
    bench.iter(|| {
        let mut cnn = TensorflowCNN::new(cnn_path);
    });
}

fn load_intent_model(bench: &mut Bencher) {
    let cnn_path = path::Path::new("../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb");
    let mut cnn = TensorflowCNN::new(cnn_path);

    let model_directory = "../data/snips-sdk-models-protobuf/tokens_classification/";
    let model_path = path::Path::new(model_directory)
        .join("BookRestaurant_CnnCrf")
        .with_extension("pbbin");

    bench.iter(|| {
        let mut tokens_classifier = ProtobufTokensClassifier::new(&model_path, &mut cnn);
    });
}

fn run_intent_model(bench: &mut Bencher) {
    let cnn_path = path::Path::new("../data/snips-sdk-models-protobuf/tokens_classification/cnn_model_quantized.pb");
    let mut cnn = TensorflowCNN::new(cnn_path);

    let model_directory = "../data/snips-sdk-models-protobuf/tokens_classification/";
    let model_path = path::Path::new(model_directory)
        .join("BookRestaurant_CnnCrf")
        .with_extension("pbbin");

    let mut tokens_classifier = ProtobufTokensClassifier::new(&model_path, &mut cnn);
    let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

    bench.iter(|| {
        tokens_classifier.run(&preprocessor_result);
    });
}

benchmark_group!(tokens_classifier, load_classifier, load_intent_model, run_intent_model);
benchmark_main!(tokens_classifier);
