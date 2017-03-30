#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use yolo::Yolo;

use queries_core::config::FileBasedAssistantConfig;

fn get_intent_parser() -> queries_core::IntentParser {
    let root_dir = queries_core::file_path("untracked");
    let assistant_config = FileBasedAssistantConfig::new(root_dir);
    queries_core::IntentParser::new(&assistant_config).yolo()
}

fn load_parser(bench: &mut Bencher) {
    bench.iter(|| {
        let _ = get_intent_parser();
    });
}

fn run_parser(bench: &mut Bencher) {
    let intent_parser = get_intent_parser();

    let text = "Book me a restaurant for two peoples at Le Chalet Savoyard";
    bench.iter(|| {
        let result = intent_parser.run_intent_classifiers(&text, 0.4);
        let _ = intent_parser.run_tokens_classifier(&text, &result[0].name).yolo();
    });
}

fn load_and_run_parser(bench: &mut Bencher) {
    let text = "Book me a restaurant for two peoples at Le Chalet Savoyard";
    bench.iter(|| {
        let intent_parser = get_intent_parser();
        let result = intent_parser.run_intent_classifiers(text, 0.4);
        let _ = intent_parser.run_tokens_classifier(text, &result[0].name).yolo();
    });
}

benchmark_group!(load, load_parser);
benchmark_group!(run, run_parser);
benchmark_group!(everything, load_and_run_parser);
benchmark_main!(load, run, everything);
