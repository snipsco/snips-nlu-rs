#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use yolo::Yolo;

use queries_core::config::{AssistantConfig, FileBasedAssistantConfig};
use queries_core::pipeline::intent_classifier::{IntentClassifier, ProtobufIntentClassifier};

fn get_classifier(classifier: &str) -> ProtobufIntentClassifier {
    let assistant_config = FileBasedAssistantConfig::new("../data2");
    let intent_config = assistant_config
        .get_intent_configuration(classifier)
        .yolo();
    ProtobufIntentClassifier::new(intent_config).yolo()
}

fn load_classifier(bench: &mut Bencher) {
    bench.iter(|| {
        let _ = get_classifier("BookRestaurant");
    });
}

fn run_classifier(bench: &mut Bencher) {
    let classifier = get_classifier("BookRestaurant");
    let preprocessor_result = queries_core::preprocess("Book me a table for two people at Le Chalet Savoyard");

    bench.iter(|| { let _ = classifier.run(&preprocessor_result); });
}

benchmark_group!(intent_classifier, load_classifier, run_classifier);
benchmark_main!(intent_classifier);
