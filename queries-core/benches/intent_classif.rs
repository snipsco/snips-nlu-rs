#[macro_use]
extern crate bencher;

extern crate snips_queries_core;

use std::{path};

use bencher::Bencher;
use snips_queries_core::preprocess;
use snips_queries_core::pipeline::intent_classifier::ProtobufIntentClassifier;
use snips_queries_core::pipeline::intent_classifier::IntentClassifier;

macro_rules! load_classifier {
    ($name:ident, $classifier:expr) => {
        fn $name(bench: &mut Bencher) {
            let model_directory = path::Path::new("../data/snips-sdk-models-protobuf/intent_classification/");
            let model_filename = model_directory.join($classifier).with_extension("pbbin");

            bench.iter(|| {
                ProtobufIntentClassifier::new(&model_filename);
            });
        }
    }
}

macro_rules! run_classifier {
    ($name:ident, $classifier:expr, $input:expr) => {
        fn $name(bench: &mut Bencher) {
            let model_directory = path::Path::new("../data/snips-sdk-models-protobuf/intent_classification/");
            let model_filename = model_directory.join($classifier).with_extension("pbbin");

            let parsed_input = preprocess($input);
            let classifier = ProtobufIntentClassifier::new(&model_filename);

            bench.iter(|| {
                classifier.run(&parsed_input);
            });
        }
    }
}

load_classifier!(load_book_restaurant, "BookRestaurant");
load_classifier!(load_place_details, "GetPlaceDetails");
run_classifier!(run_book_restaurant_coinstot, "BookRestaurant",
    "Book me a table at Coinsto Vino");
run_classifier!(run_details_quiet_luxembourg, "GetPlaceDetails",
    "What are the most quiet times to go to jardin du Luxembourg?");

benchmark_group!(load, load_book_restaurant, load_place_details);
benchmark_group!(run, run_book_restaurant_coinstot, run_details_quiet_luxembourg);
benchmark_main!(load,run);
