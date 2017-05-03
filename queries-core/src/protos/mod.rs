#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod feature {
    include!(concat!(env!("OUT_DIR"), "/protos/feature.rs"));
}

#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod slot {
    include!(concat!(env!("OUT_DIR"), "/protos/slot.rs"));
}

#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod intent_configuration {
    include!(concat!(env!("OUT_DIR"), "/protos/intent_configuration.rs"));
}

#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod model_configuration {
    include!(concat!(env!("OUT_DIR"), "/protos/model_configuration.rs"));
}

#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod regex_intent_parser_configuration {
    include!(concat!(env!("OUT_DIR"), "/protos/regex_intent_parser_configuration.rs"));
}

pub use self::regex_intent_parser_configuration::RegexIntentParserConfiguration;
pub use self::regex_intent_parser_configuration::RegexIntentModelConfiguration;

#[allow(unknown_lints)]
#[allow(clippy)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unsafe_code)]
#[allow(unused_imports)]
#[allow(unused_results)]
pub mod entity {
    include!(concat!(env!("OUT_DIR"), "/protos/entity.rs"));
}

pub use self::entity::Entity;
