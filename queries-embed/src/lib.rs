extern crate error_chain;
extern crate libc;
extern crate queries_core;

use std::ffi::{ CStr, CString };
use std::mem::transmute;
use std::sync::{ Mutex };

use errors::*;

mod errors;

pub struct Opaque(std::sync::Mutex<queries_core::IntentParser>);

#[no_mangle]
pub extern "C" fn intent_parser_create(root_dir: *const libc::c_char,
                                       client: *mut *mut Opaque) {
    let root_dir = unsafe { CStr::from_ptr(root_dir).to_str().unwrap() };

    let file_configuration = queries_core::FileConfiguration::new(root_dir);
    let intent_parser = queries_core::IntentParser::new(&file_configuration, None).unwrap();

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };
}

#[no_mangle]
pub extern "C" fn intent_parser_run_intent_classification(input: *const libc::c_char,
                                                          probability_threshold: f64,
                                                          client: *mut Opaque) {
    let input = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let client: &Opaque = unsafe { transmute(client) };

    let intent_parser = client.0.lock().unwrap();

    let results = intent_parser.run_intent_classifiers(input, probability_threshold, None);
}

#[no_mangle]
pub extern "C" fn intent_parser_run_tokens_classification(input: *const libc::c_char,
                                                          intent_name: *const libc::c_char,
                                                          client: *mut Opaque) -> CString {
    let input = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let intent_name = unsafe { CStr::from_ptr(intent_name).to_str().unwrap() };
    let client: &Opaque = unsafe { transmute(client) };

    let intent_parser = client.0.lock().unwrap();

    let result = intent_parser.run_tokens_classifier(input, intent_name).unwrap();

    panic!()
}

#[no_mangle]
pub extern "C" fn intent_parser_destroy(client: *mut Opaque) {
    unsafe {
        let _voter: Box<Opaque> = Box::from_raw(client);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        queries_core::IntentParser::new()
    }
}
