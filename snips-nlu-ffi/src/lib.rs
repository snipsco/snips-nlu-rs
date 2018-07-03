#![allow(non_camel_case_types)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate ffi_utils;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate serde_json;
extern crate snips_nlu_lib;
extern crate snips_nlu_ontology_ffi_macros;

use failure::ResultExt;

use std::ffi::CString;
use std::io::Cursor;
use std::slice;
use std::sync::Mutex;

use snips_nlu_lib::{FileBasedModel, SnipsNluEngine, ZipBasedModel};
use snips_nlu_ontology_ffi_macros::CIntentParserResult;

use ffi_utils::*;

type Result<T> = std::result::Result<T, failure::Error>;

pub struct CSnipsNluEngine(std::sync::Mutex<SnipsNluEngine>);

macro_rules! get_intent_parser {
    ($opaque:ident) => {{
        unsafe { <CSnipsNluEngine as ffi_utils::RawBorrow<CSnipsNluEngine>>::raw_borrow($opaque) }?
            .0
            .lock()
            .map_err(|e| format_err!("poisoning pointer: {}", e))?
    }};
}

generate_error_handling!(snips_nlu_engine_get_last_error);

#[no_mangle]
pub extern "C" fn snips_nlu_engine_create_from_dir(
    root_dir: *const libc::c_char,
    client: *mut *const CSnipsNluEngine,
) -> SNIPS_RESULT {
    wrap!(create_from_dir(root_dir, client))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_create_from_file(
    file_path: *const libc::c_char,
    client: *mut *const CSnipsNluEngine,
) -> SNIPS_RESULT {
    wrap!(create_from_file(file_path, client))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_create_from_zip(
    zip: *const libc::c_uchar,
    zip_size: libc::c_uint,
    client: *mut *const CSnipsNluEngine,
) -> SNIPS_RESULT {
    wrap!(create_from_zip(zip, zip_size, client))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_parse(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result: *mut *const CIntentParserResult,
) -> SNIPS_RESULT {
    wrap!(run_parse(client, input, result))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_parse_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> SNIPS_RESULT {
    wrap!(run_parse_into_json(client, input, result_json))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_destroy_string(string: *mut libc::c_char) -> SNIPS_RESULT {
    wrap!(unsafe { CString::from_raw_pointer(string) })
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_destroy_client(client: *mut CSnipsNluEngine) -> SNIPS_RESULT {
    wrap!(unsafe { CSnipsNluEngine::from_raw_pointer(client) })
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_destroy_result(
    result: *mut CIntentParserResult,
) -> SNIPS_RESULT {
    wrap!(unsafe { CIntentParserResult::from_raw_pointer(result) })
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_get_model_version(
    version: *mut *const libc::c_char,
) -> SNIPS_RESULT {
    wrap!(get_model_version(version))
}

fn create_from_dir(
    root_dir: *const libc::c_char,
    client: *mut *const CSnipsNluEngine,
) -> Result<()> {
    let root_dir = create_rust_string_from!(root_dir);

    let assistant_config = FileBasedModel::from_dir(root_dir, false)?;
    let intent_parser = SnipsNluEngine::new(assistant_config)?;

    let raw_pointer = CSnipsNluEngine(Mutex::new(intent_parser)).into_raw_pointer();
    unsafe { *client = raw_pointer };

    Ok(())
}

fn create_from_file(
    file_path: *const libc::c_char,
    client: *mut *const CSnipsNluEngine,
) -> Result<()> {
    let file_path = create_rust_string_from!(file_path);

    let assistant_config = FileBasedModel::from_path(file_path, false)?;
    let intent_parser = SnipsNluEngine::new(assistant_config)?;

    let raw_pointer = CSnipsNluEngine(Mutex::new(intent_parser)).into_raw_pointer();
    unsafe { *client = raw_pointer };

    Ok(())
}

fn create_from_zip(
    zip: *const libc::c_uchar,
    zip_size: libc::c_uint,
    client: *mut *const CSnipsNluEngine,
) -> Result<()> {
    let slice = unsafe { slice::from_raw_parts(zip, zip_size as usize) };
    let reader = Cursor::new(slice.to_owned());

    let assistant_config = ZipBasedModel::new(reader, false)?;
    let intent_parser = SnipsNluEngine::new(assistant_config)?;

    let raw_pointer = CSnipsNluEngine(Mutex::new(intent_parser)).into_raw_pointer();
    unsafe { *client = raw_pointer };

    Ok(())
}

fn run_parse(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result: *mut *const CIntentParserResult,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(&input, None)?;
    let raw_pointer = CIntentParserResult::from(results).into_raw_pointer();

    unsafe { *result = raw_pointer };

    Ok(())
}

fn run_parse_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(&input, None)?;

    point_to_string(result_json, serde_json::to_string(&results)?)
}

fn get_model_version(version: *mut *const libc::c_char) -> Result<()> {
    point_to_string(version, snips_nlu_lib::MODEL_VERSION.to_string())
}
