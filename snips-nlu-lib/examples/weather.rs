extern crate serde_json;
extern crate snips_nlu_lib;

use std::env;

use snips_nlu_lib::{FromPath, SnipsNluEngine};

fn main() {
    let args: Vec<String> = env::args().collect();
    let model_dir = &args[1];
    let query = &args[2];
    let nlu_engine = SnipsNluEngine::from_path(model_dir).unwrap();

    let result = nlu_engine.parse(query, None).unwrap();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
