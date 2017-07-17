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

const ASSISTANT_ZIP_ENV: &str = "SNIPS_QUERIES_BENCH_ASSISTANT_ZIP";
const ASSISTANT_DIR_ENV: &str = "SNIPS_QUERIES_BENCH_ASSISTANT_DIR";
const SENTENCE_ENV: &str = "SNIPS_QUERIES_BENCH_SENTENCE";

fn load_nlu_engine() -> SnipsNLUEngine {
    if env::var(ASSISTANT_ZIP_ENV).is_ok() && env::var(ASSISTANT_DIR_ENV).is_ok() {
        panic!("{} and {} env vars are exclusive. Please use only one of both", ASSISTANT_ZIP_ENV, ASSISTANT_DIR_ENV);
    }

    if let Ok(assistant_zip) = env::var(ASSISTANT_ZIP_ENV) {
        let file = fs::File::open(file_path(&assistant_zip)).yolo();
        let assistant = ZipBasedConfiguration::new(file).yolo();
        SnipsNLUEngine::new(assistant).yolo()
    } else if let Ok(assistant_directory) = env::var(ASSISTANT_DIR_ENV) {
        let assistant = FileBasedConfiguration::new(file_path(&assistant_directory)).yolo();
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
    let sentence = env::var(SENTENCE_ENV)
        .map_err(|_| format!("{} env var not defined", SENTENCE_ENV))
        .yolo();

    b.iter(|| {
        let _ = nlu_engine.parse(&sentence, None);
    });
}

benchmark_group!(load, nlu_loading);
benchmark_group!(run, nlu_parsing);

benchmark_main!(load, run);
