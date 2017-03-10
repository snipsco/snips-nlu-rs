extern crate libc;
extern crate queries_core;
#[macro_use]
extern crate lazy_static;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::sync::{Mutex};

extern crate serde_json;


#[repr(C)]
pub struct Opaque(std::sync::Mutex<queries_core::IntentParser>);


#[repr(C)]
pub enum QUERIESRESULT {
    KO = 0,
    OK = 1,
}


lazy_static! {
    static ref LAST_ERROR:std::sync::Mutex<String> = std::sync::Mutex::new("".to_string());
}

macro_rules! ctry {
    ( $e:expr ) => { match $e {
        Ok(ok) => ok,
        Err(e) => {
            let msg = format!("{:?}", e);
            match LAST_ERROR.lock() {
                Ok(mut guard) => *guard = msg,
                Err(_) => () /* curl up and cry */
            }
            return QUERIESRESULT::KO;
        }
    }}
}


#[no_mangle]
pub extern "C" fn intent_parser_create(root_dir: *const libc::c_char, client: *mut *mut Opaque) -> QUERIESRESULT {
    let root_dir = ctry!(unsafe { CStr::from_ptr(root_dir).to_str() });
    let file_configuration = queries_core::FileConfiguration::new(root_dir);
    let intent_parser = ctry!(queries_core::IntentParser::new(&file_configuration, None));

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn intent_parser_run_intent_classification(client: *mut Opaque,
                                                          input: *const libc::c_char,
                                                          probability_threshold: libc::c_float,
                                                          result_json: *mut *mut libc::c_char) -> QUERIESRESULT {
    let input = ctry!(unsafe { CStr::from_ptr(input).to_str() });
    let client: &Opaque = unsafe { transmute(client) };

    let intent_parser = ctry!(client.0.lock());

    let results = intent_parser.run_intent_classifiers(input, probability_threshold as f64, None);

    unsafe { *result_json = to_ptr(ctry!(serde_json::to_string(&results))) };

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn intent_parser_run_tokens_classification(client: *mut Opaque,
                                                          input: *const libc::c_char,
                                                          intent_name: *const libc::c_char,
                                                          result_json: *mut *mut libc::c_char) -> QUERIESRESULT {
    let input = ctry!(unsafe { CStr::from_ptr(input).to_str() });
    let intent_name = ctry!(unsafe { CStr::from_ptr(intent_name).to_str() });
    let client: &Opaque = unsafe { transmute(client) };

    let intent_parser = ctry!(client.0.lock());

    let result = ctry!(intent_parser.run_tokens_classifier(input, intent_name));

    unsafe { *result_json = to_ptr(ctry!(serde_json::to_string(&result))) };

    QUERIESRESULT::OK
}

/// Convert a Rust string to a native string
unsafe fn to_ptr(string: String) -> *mut libc::c_char {
    let cs = CString::new(string.as_bytes()).unwrap();
    cs.into_raw()
}


#[no_mangle]
pub extern "C" fn intent_parser_destroy_string(string: *mut libc::c_char) -> QUERIESRESULT {
    unsafe {
        let _string: CString = CString::from_raw(string);
    }

    QUERIESRESULT::OK
}


#[no_mangle]
pub extern "C" fn intent_parser_destroy_client(client: *mut Opaque) -> QUERIESRESULT {
    unsafe {
        let _parser: Box<Opaque> = Box::from_raw(client);
    }

    QUERIESRESULT::OK
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        queries_core::IntentParser::new()
    }
}
