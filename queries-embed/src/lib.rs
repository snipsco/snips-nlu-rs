extern crate queries_core;
extern crate snips_queries_ontology;
extern crate snips_queries_ontology_ffi;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate error_chain;
extern crate serde_json;

use std::ffi::{CStr, CString};
use std::sync::Mutex;
use std::slice;
use std::io::Cursor;

use queries_core::{SnipsNluEngine, FileBasedConfiguration, ZipBasedConfiguration};
use snips_queries_ontology_ffi::{CIntentParserResult, CTaggedEntityList};

lazy_static! {
    static ref LAST_ERROR: Mutex<String> = Mutex::new("".to_string());
}

mod errors {
    error_chain! {
        links {
            SnipsQueries(::queries_core::Error, ::queries_core::ErrorKind);
            SnipsQueriesOntology(::snips_queries_ontology::OntologyError, ::snips_queries_ontology::OntologyErrorKind);
        }

        foreign_links {
            Io(::std::io::Error);
            Serde(::serde_json::Error);
            Utf8Error(::std::str::Utf8Error);
            NulError(::std::ffi::NulError);
        }
    }

    impl<T> ::std::convert::From<::std::sync::PoisonError<T>> for Error {
        fn from(pe: ::std::sync::PoisonError<T>) -> Error {
            format!("Poisoning error: {:?}", pe).into()
        }
    }
}

use errors::*;

#[repr(C)]
pub struct Opaque(std::sync::Mutex<SnipsNluEngine>);

#[repr(C)]
#[derive(Debug)]
pub enum QUERIESRESULT {
    KO = 0,
    OK = 1,
}

macro_rules! wrap {
    ($e:expr) => { match $e {
        Ok(_) => { QUERIESRESULT::OK }
        Err(e) => {
            use error_chain::ChainedError;
            let msg = e.display_chain().to_string();
            eprintln!("{}", msg);
            match LAST_ERROR.lock() {
                Ok(mut guard) => *guard = msg,
                Err(_) => () /* curl up and cry */
            }
            QUERIESRESULT::KO
        }
    }}
}

macro_rules! get_intent_parser {
    ($opaque:ident) => {{
        let client: &Opaque = unsafe { &*$opaque };
        client.0.lock()?
    }};
}

macro_rules! get_str {
    ($pointer:ident) => {{
        unsafe { CStr::from_ptr($pointer) }.to_str()?
    }};
}

#[no_mangle]
pub extern "C" fn nlu_engine_create_from_dir(root_dir: *const libc::c_char,
                                             client: *mut *const Opaque)
                                             -> QUERIESRESULT {
    wrap!(create_from_dir(root_dir, client))
}

#[no_mangle]
pub extern "C" fn nlu_engine_create_from_zip(zip: *const libc::c_uchar,
                                             zip_size: libc::c_uint,
                                             client: *mut *const Opaque)
                                             -> QUERIESRESULT {
    wrap!(create_from_zip(zip, zip_size, client))
}

#[no_mangle]
pub extern "C" fn nlu_engine_run_parse(client: *const Opaque,
                                       input: *const libc::c_char,
                                       result: *mut *const CIntentParserResult)
                                       -> QUERIESRESULT {
    wrap!(run_parse(client, input, result))
}

#[no_mangle]
pub extern "C" fn nlu_engine_run_parse_into_json(client: *const Opaque,
                                                 input: *const libc::c_char,
                                                 result_json: *mut *const libc::c_char)
                                                 -> QUERIESRESULT {
    wrap!(run_parse_into_json(client, input, result_json))
}

#[no_mangle]
pub extern "C" fn nlu_engine_run_tag(client: *const Opaque,
                                     input: *const libc::c_char,
                                     intent: *const libc::c_char,
                                     result: *mut *const CTaggedEntityList)
                                     -> QUERIESRESULT {
    wrap!(run_tag(client, input, intent, result))
}

#[no_mangle]
pub extern "C" fn nlu_engine_get_last_error(error: *mut *const libc::c_char) -> QUERIESRESULT {
    wrap!(get_last_error(error))
}

#[no_mangle]
pub extern "C" fn nlu_engine_destroy_string(string: *mut libc::c_char) -> QUERIESRESULT {
    unsafe {
        let _: CString = CString::from_raw(string);
    }

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn nlu_engine_destroy_client(client: *mut Opaque) -> QUERIESRESULT {
    unsafe {
        let _: Box<Opaque> = Box::from_raw(client);
    }

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn nlu_engine_destroy_result(result: *mut CIntentParserResult) -> QUERIESRESULT {
    unsafe {
        let _: Box<CIntentParserResult> = Box::from_raw(result);
    }

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn nlu_engine_destroy_tagged_entity_list(result: *mut CTaggedEntityList) -> QUERIESRESULT {
    unsafe {
        let _: Box<CTaggedEntityList> = Box::from_raw(result);
    }

    QUERIESRESULT::OK
}

#[no_mangle]
pub extern "C" fn nlu_engine_get_model_version(version: *mut *const libc::c_char) -> QUERIESRESULT {
    wrap!(get_model_version(version))
}

fn create_from_dir(root_dir: *const libc::c_char, client: *mut *const Opaque) -> Result<()> {
    let root_dir = get_str!(root_dir);

    let assistant_config = FileBasedConfiguration::new(root_dir)?;
    let intent_parser = SnipsNluEngine::new(assistant_config)?;

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };

    Ok(())
}

fn create_from_zip(zip: *const libc::c_uchar,
                   zip_size: libc::c_uint,
                   client: *mut *const Opaque)
                   -> Result<()> {
    let slice = unsafe { slice::from_raw_parts(zip, zip_size as usize) };
    let reader = Cursor::new(slice.to_owned());

    let assistant_config = ZipBasedConfiguration::new(reader)?;
    let intent_parser = SnipsNluEngine::new(assistant_config)?;

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };

    Ok(())
}

fn run_parse(client: *const Opaque,
             input: *const libc::c_char,
             result: *mut *const CIntentParserResult)
             -> Result<()> {
    let input = get_str!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(input, None)?;
    let b = Box::new(CIntentParserResult::from(results)?);

    unsafe { *result = Box::into_raw(b) as *const CIntentParserResult }

    Ok(())
}

fn run_parse_into_json(client: *const Opaque,
                       input: *const libc::c_char,
                       result_json: *mut *const libc::c_char)
                       -> Result<()> {
    let input = get_str!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(input, None)?;

    point_to_string(result_json, serde_json::to_string(&results)?)
}

fn run_tag(client: *const Opaque,
           input: *const libc::c_char,
           intent: *const libc::c_char,
           result: *mut *const CTaggedEntityList)
           -> Result<()> {
    let input = get_str!(input);
    let intent = get_str!(intent);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.tag(input, intent, None)?;
    let b = Box::new(CTaggedEntityList::from(results)?);

    unsafe { *result = Box::into_raw(b) as *const CTaggedEntityList }

    Ok(())
}

fn get_last_error(error: *mut *const libc::c_char) -> Result<()> {
    point_to_string(error, LAST_ERROR.lock()?.clone())
}

fn get_model_version(version: *mut *const libc::c_char) -> Result<()> {
    point_to_string(version, SnipsNluEngine::model_version().to_string())
}

fn point_to_string(pointer: *mut *const libc::c_char, string: String) -> Result<()> {
    let cs = CString::new(string.as_bytes())?;
    unsafe { *pointer = cs.into_raw() }
    Ok(())
}
