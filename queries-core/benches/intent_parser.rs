#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use yolo::Yolo;

use queries_core::FileBasedAssistantConfig;
use queries_core::IntentParser;

fn get_intent_parser() -> queries_core::DeepIntentParser {
    let root_dir = queries_core::file_path("untracked");
    let assistant_config = FileBasedAssistantConfig::new(root_dir).yolo();
    queries_core::DeepIntentParser::new(&assistant_config).yolo()
}

fn load_parser(bench: &mut Bencher) {
    bench.iter(|| {
        let _ = get_intent_parser();
    });
}

macro_rules! run_parser {
    ($name:ident, $input:expr) => {
        fn $name(bench: &mut Bencher) {
            let intent_parser = get_intent_parser();

            bench.iter(|| {
                let result = intent_parser.get_intent($input, 0.4).yolo();
                let _ = intent_parser.get_entities($input, &result[0].name).yolo();
            });
        }
    }
}

run_parser!(run_book_restaurant,
"Book me a table for four people at Le Chalet Savoyard tonight");
run_parser!(run_get_weather,
"What will be the weather tomorrow in Paris ?");
run_parser!(run_play_music,
"Give me some psychedelic hip-hop please");

benchmark_group!(load, load_parser);
benchmark_group!(run, run_book_restaurant, run_get_weather, run_play_music);

benchmark_main!(load, run);
