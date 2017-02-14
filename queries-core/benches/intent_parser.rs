#[macro_use]
extern crate bencher;

extern crate snips_queries_core;

use bencher::Bencher;
use snips_queries_core::FileConfiguration;
use snips_queries_core::IntentParser;

fn load_intent_parser(bench: &mut Bencher) {
    let file_configuration = FileConfiguration::default();

    bench.iter(|| {
        let _ = IntentParser::new(&file_configuration, &["BookRestaurant"]);
    });
}

fn run_intent_classifications(bench: &mut Bencher) {
    let file_configuration = FileConfiguration::default();
    let intent_parser = IntentParser::new(&file_configuration, &["BookRestaurant"]);

    bench.iter(|| {
        let _ = intent_parser.run_intent_classifiers("Book me a restaurant for two peoples at Le Chalet Savoyard", 0.4);
    });
}

benchmark_group!(load, load_intent_parser);
benchmark_group!(run, run_intent_classifications);
benchmark_main!(load,run);
