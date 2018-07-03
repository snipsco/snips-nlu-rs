extern crate serde_json;
extern crate snips_nlu_lib;

use std::env;

use snips_nlu_lib::{FileBasedModel, SnipsNluEngine};

fn main() {
    let args: Vec<String> = env::args().collect();
    let model_file = &args[1];
    let query = &args[2];
    let configuration = match FileBasedModel::from_path(model_file, false) {
        Ok(conf) => conf,
        Err(e) => panic!(format!("{}", e)),
    };
    let nlu_engine = SnipsNluEngine::new(configuration).unwrap();

    let result = nlu_engine.parse(query, None).unwrap();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
