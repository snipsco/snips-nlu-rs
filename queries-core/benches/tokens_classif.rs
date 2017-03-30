#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use yolo::Yolo;

use queries_core::config::{AssistantConfig, FileBasedAssistantConfig};
use queries_core::pipeline::tokens_classifier::{TokensClassifier, ProtobufTokensClassifier};

fn get_classifier(classifier: &str) -> ProtobufTokensClassifier {
    let root_dir = queries_core::file_path("untracked");
    let assistant_config = FileBasedAssistantConfig::new(root_dir);
    let intent_config = assistant_config
        .get_intent_configuration(classifier)
        .yolo();
    ProtobufTokensClassifier::new(intent_config).yolo()
}

fn load_classifier(bench: &mut Bencher) {
    bench.iter(|| {
        let _ = get_classifier("BookRestaurant");
    });
}

fn run_classifier(bench: &mut Bencher) {
    let tokens_classifier = get_classifier("BookRestaurant");
    let preprocessor_result = queries_core::preprocess("Book me a table for two people at Le Chalet Savoyard");

    bench.iter(|| { let _ = tokens_classifier.run(&preprocessor_result); });
}

benchmark_group!(tokens_classifier, load_classifier, run_classifier);
benchmark_main!(tokens_classifier);
