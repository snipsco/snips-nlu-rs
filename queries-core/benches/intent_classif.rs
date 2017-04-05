#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use yolo::Yolo;

use queries_core::config::{AssistantConfig, FileBasedAssistantConfig};
use queries_core::pipeline::intent_classifier::{IntentClassifier, ProtobufIntentClassifier};

fn get_intent_classifier(classifier: &str) -> ProtobufIntentClassifier {
    let root_dir = queries_core::file_path("untracked");
    let assistant_config = FileBasedAssistantConfig::new(root_dir);
    let intent_config = assistant_config
        .get_intent_configuration(classifier)
        .yolo();
    ProtobufIntentClassifier::new(intent_config).yolo()
}

fn load_intent_classifier(bench: &mut Bencher) {
    bench.iter(|| {
        let _ = get_intent_classifier("BookRestaurant");
    });
}

fn run_intent_classifier(bench: &mut Bencher) {
    let classifier = get_intent_classifier("BookRestaurant");
    let preprocessor_result = queries_core::preprocess(
        "Book me a table for four people at Le Chalet Savoyard tonight",
        r#"[{"end_index": 24, "value": "four", "start_index": 20, "entity": "%NUMBER%"}, {"end_index": 61, "value": "tonight", "start_index": 54, "entity": "%TIME_INTERVAL%"}]"#)
        .yolo();

    bench.iter(|| { let _ = classifier.run(&preprocessor_result); });
}

benchmark_group!(intent_classifier, load_intent_classifier, run_intent_classifier);
benchmark_main!(intent_classifier);
