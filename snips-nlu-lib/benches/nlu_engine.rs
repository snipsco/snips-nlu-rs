#[macro_use]
extern crate bencher;
extern crate snips_nlu_lib;
extern crate dinghy_test;

use std::env;

use bencher::Bencher;
use snips_nlu_lib::*;

const ENGINE_DIR_ENV: &str = "SNIPS_NLU_BENCH_ENGINE_DIR";
const SENTENCE_ENV: &str = "SNIPS_NLU_BENCH_SENTENCE";

fn file_path(filename: &str) -> ::std::path::PathBuf {
    dinghy_test::try_test_file_path("data")
        .unwrap_or_else(|| "../data".into())
        .join(filename)
}

fn load_nlu_engine() -> SnipsNluEngine {
    let engine_path = if let Ok(engine_directory) = env::var(ENGINE_DIR_ENV) {
        file_path(&engine_directory)
    } else {
        file_path("untracked")
    };

    SnipsNluEngine::from_path(engine_path).unwrap()
}

fn nlu_loading(b: &mut Bencher) {
    b.iter(|| {
        clear_resources();
        let _ = load_nlu_engine();
    });
}

fn nlu_parsing(b: &mut Bencher) {
    let nlu_engine = load_nlu_engine();
    let sentence = env::var(SENTENCE_ENV)
        .map_err(|_| format!("{} env var not defined", SENTENCE_ENV))
        .unwrap();

    b.iter(|| {
        let _ = nlu_engine.parse(&sentence, None);
    });
}

benchmark_group!(load, nlu_loading);
benchmark_group!(run, nlu_parsing);

benchmark_main!(load, run);
