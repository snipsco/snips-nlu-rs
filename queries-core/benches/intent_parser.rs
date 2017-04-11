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

macro_rules! run_parser {
    ($name:ident, $input:expr, $json:expr) => {
        fn $name(bench: &mut Bencher) {
            let intent_parser = get_intent_parser();

            bench.iter(|| {
                let result = intent_parser.run_intent_classifiers($input, 0.4, $json).yolo();
                let _ = intent_parser.run_tokens_classifier($input, &result[0].name, $json).yolo();
            });
        }
    }
}

macro_rules! load_and_run_parser {
    ($name:ident, $input:expr, $json:expr) => {
        fn $name(bench: &mut Bencher) {
            bench.iter(|| {
                let intent_parser = get_intent_parser();
                let result = intent_parser.run_intent_classifiers($input, 0.4, $json).yolo();
                let _ = intent_parser.run_tokens_classifier($input, &result[0].name, $json).yolo();
            });
        }
    }
}

run_parser!(run_book_restaurant,
"Book me a table for four people at Le Chalet Savoyard tonight",
r#"[{"end_index": 24, "value": "four", "start_index": 20, "entity": "%NUMBER%"}, {"end_index": 61, "value": "tonight", "start_index": 54, "entity": "%TIME_INTERVAL%"}]"#);
run_parser!(run_get_weather,
"What will be the weather tomorrow in Paris ?",
r#"[{"end_index": 33, "value": "tomorrow", "start_index": 25, "entity": "%TIME%"}]"#);
run_parser!(run_play_music,
"Give me some psychedelic hip-hop please",
r#"[]"#);

benchmark_group!(load, load_parser);
benchmark_group!(run, run_book_restaurant, run_get_weather, run_play_music);
//benchmark_group!(everything, load_and_run_book_restaurant, load_and_run_get_weather);

benchmark_main!(load, run/*, everything*/);
