#[macro_use]
extern crate bencher;
extern crate queries_core;
extern crate yolo;

use bencher::Bencher;
use queries_core::{SnipsNLUEngine, FileBasedConfiguration};
use queries_core::file_path;
use yolo::Yolo;

fn load_nlu_engine(b: &mut Bencher) {
    b.iter(|| {
        let assistant = FileBasedConfiguration::new(file_path("untracked")).yolo();
        let _ = SnipsNLUEngine::new(assistant).yolo();
    });
}

fn run_nlu_engine(b: &mut Bencher) {
    let assistant = FileBasedConfiguration::new(file_path("untracked")).yolo();
    let nlu_engine = SnipsNLUEngine::new(assistant).yolo();

    b.iter(|| {
        let _ = nlu_engine.parse("What's the weather in Berlin the day after tomorrow?", None);
    });
}

benchmark_group!(load, load_nlu_engine);
benchmark_group!(run, run_nlu_engine);

benchmark_main!(load, run);
