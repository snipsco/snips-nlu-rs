extern crate serde_json;
extern crate snips_nlu_lib;

use std::io::prelude::*;
use std::fs::File;
use std::env;

use snips_nlu_lib::{NluEngineConfiguration, SnipsNluEngine};

fn main() {
    let args: Vec<String> = env::args().collect();
    let modelfilename = &args[1];
    let query = &args[2];
    let mut file = File::open(modelfilename).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");

    let configuration: NluEngineConfiguration = serde_json::from_str(&contents).unwrap();
    let nlu_engine = SnipsNluEngine::new(configuration).unwrap();

    let result = nlu_engine.parse(query, None).unwrap();

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
