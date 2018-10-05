extern crate clap;
extern crate serde_json;
extern crate snips_nlu_lib;

use clap::{Arg, App};
use snips_nlu_lib::SnipsNluEngine;
use std::io;
use std::io::Write;

fn main() {
    let matches = App::new("snips-nlu-parse")
        .about("Snips NLU interactive CLI for parsing intents")
        .arg(Arg::with_name("NLU_ENGINE_DIR")
            .required(true)
            .takes_value(true)
            .index(1)
            .help("path to the trained nlu engine directory"))
        .get_matches();
    let engine_dir = matches.value_of("NLU_ENGINE_DIR").unwrap();

    println!("\nLoading the nlu engine...");
    let engine = SnipsNluEngine::from_path(engine_dir).unwrap();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let result = engine.parse(query.trim(), None).unwrap();
        let result_json = serde_json::to_string_pretty(&result).unwrap();
        println!("{}", result_json);
    }
}
