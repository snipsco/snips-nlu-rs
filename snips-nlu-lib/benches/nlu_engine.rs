#[macro_use]
extern crate bencher;
extern crate snips_nlu_lib;

use std::{env, fs};

use bencher::Bencher;
use snips_nlu_lib::file_path;
use snips_nlu_lib::{FileBasedConfiguration, SnipsNluEngine, ZipBasedConfiguration};

const ASSISTANT_ZIP_ENV: &str = "SNIPS_NLU_BENCH_ASSISTANT_ZIP";
const ASSISTANT_DIR_ENV: &str = "SNIPS_NLU_BENCH_ASSISTANT_DIR";
const BYPASS_MODEL_VERSION_ENV: &str = "SNIPS_NLU_BENCH_BYPASS_MODEL_VERSION";
const SENTENCE_ENV: &str = "SNIPS_NLU_BENCH_SENTENCE";

fn load_nlu_engine() -> SnipsNluEngine {
    if env::var(ASSISTANT_ZIP_ENV).is_ok() && env::var(ASSISTANT_DIR_ENV).is_ok() {
        panic!(
            "{} and {} env vars are exclusive. Please use only one of both",
            ASSISTANT_ZIP_ENV, ASSISTANT_DIR_ENV
        );
    }

    let bypass_model_version_check = if let Ok(value) = env::var(BYPASS_MODEL_VERSION_ENV) {
        if let Ok(int_value) = value.parse::<i32>() {
            int_value > 0
        } else {
            true
        }
    } else {
        false
    };

    if let Ok(assistant_zip) = env::var(ASSISTANT_ZIP_ENV) {
        let file = fs::File::open(file_path(&assistant_zip)).yolo();
        let assistant = ZipBasedConfiguration::new(file, bypass_model_version_check).yolo();
        SnipsNluEngine::new(assistant).yolo()
    } else if let Ok(assistant_directory) = env::var(ASSISTANT_DIR_ENV) {
        let assistant = FileBasedConfiguration::from_path(
            file_path(&assistant_directory),
            bypass_model_version_check,
        ).yolo();
        SnipsNluEngine::new(assistant).yolo()
    } else {
        let assistant =
            FileBasedConfiguration::from_path(file_path("untracked"), bypass_model_version_check).yolo();
        SnipsNluEngine::new(assistant).yolo()
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
