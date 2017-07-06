#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use std::{env, fs};

use bencher::Bencher;
use queries_core::{
    SnipsNLUEngine,
    FileBasedConfiguration,
    ZipBasedConfiguration};
use queries_core::file_path;
use yolo::Yolo;

fn load_nlu_engine() -> SnipsNLUEngine {
    if let Ok(assistant_directory) = env::var("SNIPS_QUERIES_BENCH_ASSISTANT_DIR") {
        let assistant = FileBasedConfiguration::new(file_path(&assistant_directory)).yolo();
        SnipsNLUEngine::new(assistant).yolo()
    } else if let Ok(assistant_zip) = env::var("SNIPS_QUERIES_BENCH_ASSISTANT_ZIP") {
        let file = fs::File::open(file_path(&assistant_zip)).yolo();
        let assistant = ZipBasedConfiguration::new(file).yolo();
        SnipsNLUEngine::new(assistant).yolo()
    } else {
        let assistant = FileBasedConfiguration::new(file_path("untracked")).yolo();
        SnipsNLUEngine::new(assistant).yolo()
    }
}

fn nlu_loading(b: &mut Bencher) {
    b.iter(|| {
        let _ = load_nlu_engine();
    });
}

fn nlu_parsing(b: &mut Bencher) {
    let nlu_engine = load_nlu_engine();
    let sentence = env::var("SNIPS_QUERIES_BENCH_SENTENCE")
        .map_err(|_| "SNIPS_QUERIES_BENCH_SENTENCE env var not defined")
        .yolo();

    b.iter(|| {
        let _ = nlu_engine.parse(&sentence, None);
    });
}

benchmark_group!(load, nlu_loading);
benchmark_group!(run, nlu_parsing);

benchmark_main!(load, run);
