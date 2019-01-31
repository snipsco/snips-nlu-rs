extern crate clap;
extern crate serde_json;
extern crate snips_nlu_lib;

use clap::{App, Arg};
use snips_nlu_lib::SnipsNluEngine;
use std::io;
use std::io::Write;

fn main() {
    let matches = App::new("snips-nlu-parse")
        .about("Snips NLU interactive CLI for parsing intents")
        .arg(
            Arg::with_name("NLU_ENGINE_DIR")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("path to the trained nlu engine directory"),
        )
        .arg(
            Arg::with_name("top_intents")
                .long("top_intents")
                .short("t")
                .required(false)
                .takes_value(false)
                .help("option flag to use in order to parse with the `get_intents` API"),
        )
        .get_matches();
    let engine_dir = matches.value_of("NLU_ENGINE_DIR").unwrap();
    let top_intents = matches.is_present("top_intents");

    println!("\nLoading the nlu engine...");
    let engine = SnipsNluEngine::from_path(engine_dir).unwrap();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let result_json = if top_intents {
            let result = engine.get_intents(query.trim()).unwrap();
            serde_json::to_string_pretty(&result).unwrap()
        } else {
            let result = engine.parse(query.trim(), None, None).unwrap();
            serde_json::to_string_pretty(&result).unwrap()
        };
        println!("{}", result_json);
    }
}
