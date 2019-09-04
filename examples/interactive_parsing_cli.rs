extern crate clap;
extern crate env_logger;
extern crate serde_json;
extern crate snips_nlu_lib;

use clap::{App, Arg};
use snips_nlu_lib::SnipsNluEngine;
use std::io;
use std::io::Write;

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .init();

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
            Arg::with_name("intents_alternatives")
                .short("i")
                .long("--intents-alternatives")
                .takes_value(true)
                .help("number of alternative parsing results to return in the output"),
        )
        .arg(
            Arg::with_name("slots_alternatives")
                .short("s")
                .long("--slots-alternatives")
                .takes_value(true)
                .help("number of alternative slot values to return along with each extracted slot"),
        )
        .get_matches();
    let engine_dir = matches.value_of("NLU_ENGINE_DIR").unwrap();
    let intents_alternatives = matches
        .value_of("intents_alternatives")
        .map(|v| v.to_string().parse::<usize>().unwrap())
        .unwrap_or(0);
    let slots_alternatives = matches
        .value_of("slots_alternatives")
        .map(|v| v.to_string().parse::<usize>().unwrap())
        .unwrap_or(0);

    println!("\nLoading the nlu engine...");
    let engine = SnipsNluEngine::from_path(engine_dir).unwrap();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let result = engine
            .parse_with_alternatives(
                query.trim(),
                None,
                None,
                intents_alternatives,
                slots_alternatives,
            )
            .unwrap();
        let result_json = serde_json::to_string_pretty(&result).unwrap();
        println!("{}", result_json);
    }
}
