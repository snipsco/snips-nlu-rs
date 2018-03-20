extern crate serde_json;
extern crate snips_nlu_lib;

use std::env;
use std::path::Path;

use snips_nlu_lib::{FileBasedConfiguration, SnipsNluEngine};

fn main() {
    let args: Vec<String> = env::args().collect();
    let model_directory = &args[1];
    let query = &args[2];
    let root_dir_path = Path::new(model_directory);
    let configuration = match FileBasedConfiguration::new(root_dir_path, false) {
        Ok(conf) => conf,
        Err(e) => panic!(format!("{}", e)),
    };
    let nlu_engine = SnipsNluEngine::new(configuration).unwrap();

    let result = nlu_engine.parse(query, None).unwrap();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
