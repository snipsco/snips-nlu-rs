#[macro_use]
extern crate bencher;

extern crate snips_queries_core;

use bencher::Bencher;
use snips_queries_core::FileConfiguration;
use snips_queries_core::pipeline::tokens_classifier::ProtobufTokensClassifier;
use snips_queries_core::pipeline::tokens_classifier::TokensClassifier;
use snips_queries_core::preprocess;

fn load_classifier(bench: &mut Bencher) {
    let file_configuration = FileConfiguration::default();
    let model_name = "BookRestaurant_bookRestaurant";
    let cnn_name = "Cnn_BookRestaurant_bookRestaurant";

    bench.iter(|| {
        let _ = ProtobufTokensClassifier::new(&file_configuration, &model_name, &cnn_name);
    });
}

fn run_intent_model(bench: &mut Bencher) {
    let file_configuration = FileConfiguration::default();
    let model_name = "BookRestaurant_bookRestaurant";
    let cnn_name = "Cnn_BookRestaurant_bookRestaurant";

    let tokens_classifier = ProtobufTokensClassifier::new(&file_configuration, &model_name, &cnn_name).unwrap();

    let preprocessor_result = preprocess("Book me a table for two people at Le Chalet Savoyard");

    bench.iter(|| {
        let _ = tokens_classifier.run(&preprocessor_result);
    });
}

benchmark_group!(tokens_classifier, load_classifier, run_intent_model);
benchmark_main!(tokens_classifier);
