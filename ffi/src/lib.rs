#![allow(non_camel_case_types)]

extern crate ffi_utils;
extern crate snips_nlu_ontology_ffi_macros;

use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::slice;
use std::sync::Mutex;

use failure::{format_err, ResultExt};
use ffi_utils::*;
use snips_nlu_lib::SnipsNluEngine;
use snips_nlu_ontology_ffi_macros::{CIntentClassifierResultArray, CIntentParserResult, CSlotList};

type Result<T> = std::result::Result<T, failure::Error>;

pub struct CSnipsNluEngine(std::sync::Mutex<SnipsNluEngine>);

macro_rules! get_nlu_engine {
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
    intents_whitelist: *const CStringArray,
    intents_blacklist: *const CStringArray,
    result: *mut *const CIntentParserResult,
) -> SNIPS_RESULT {
    wrap!(run_parse(
        client,
        input,
        intents_whitelist,
        intents_blacklist,
        result
    ))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_get_slots(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intent: *const libc::c_char,
    result: *mut *const CSlotList,
) -> SNIPS_RESULT {
    wrap!(run_get_slots(client, input, intent, result))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_get_intents(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result: *mut *const CIntentClassifierResultArray,
) -> SNIPS_RESULT {
    wrap!(run_get_intents(client, input, result))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_parse_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intents_whitelist: *const CStringArray,
    intents_blacklist: *const CStringArray,
    result_json: *mut *const libc::c_char,
) -> SNIPS_RESULT {
    wrap!(run_parse_into_json(
        client,
        input,
        intents_whitelist,
        intents_blacklist,
        result_json
    ))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_get_slots_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intent: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> SNIPS_RESULT {
    wrap!(run_get_slots_into_json(client, input, intent, result_json))
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_run_get_intents_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> SNIPS_RESULT {
    wrap!(run_get_intents_into_json(client, input, result_json))
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
pub extern "C" fn snips_nlu_engine_destroy_slots(result: *mut CSlotList) -> SNIPS_RESULT {
    wrap!(unsafe { CSlotList::from_raw_pointer(result) })
}

#[no_mangle]
pub extern "C" fn snips_nlu_engine_destroy_intent_classifier_results(
    result: *mut CIntentClassifierResultArray,
) -> SNIPS_RESULT {
    wrap!(unsafe { CIntentClassifierResultArray::from_raw_pointer(result) })
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

    let nlu_engine = SnipsNluEngine::from_path(root_dir)?;

    let raw_pointer = CSnipsNluEngine(Mutex::new(nlu_engine)).into_raw_pointer();
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
    let nlu_engine = SnipsNluEngine::from_zip(reader)?;
    let raw_pointer = CSnipsNluEngine(Mutex::new(nlu_engine)).into_raw_pointer();
    unsafe { *client = raw_pointer };

    Ok(())
}

fn run_parse(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intents_whitelist: *const CStringArray,
    intents_blacklist: *const CStringArray,
    result: *mut *const CIntentParserResult,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let nlu_engine = get_nlu_engine!(client);

    let opt_whitelist: Option<Vec<_>> = if !intents_whitelist.is_null() {
        Some(unsafe { convert_to_rust_vec(intents_whitelist)? })
    } else {
        None
    };
    let opt_blacklist: Option<Vec<_>> = if !intents_blacklist.is_null() {
        Some(unsafe { convert_to_rust_vec(intents_blacklist)? })
    } else {
        None
    };

    let results = nlu_engine.parse(&input, opt_whitelist, opt_blacklist)?;
    let raw_pointer = CIntentParserResult::from(results).into_raw_pointer();

    unsafe { *result = raw_pointer };

    Ok(())
}

fn run_get_slots(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intent: *const libc::c_char,
    result: *mut *const CSlotList,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let intent = create_rust_string_from!(intent);
    let nlu_engine = get_nlu_engine!(client);

    let slots = nlu_engine.get_slots(&input, &intent)?;
    let raw_pointer = CSlotList::from(slots).into_raw_pointer();

    unsafe { *result = raw_pointer };

    Ok(())
}

fn run_get_intents(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result: *mut *const CIntentClassifierResultArray,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let nlu_engine = get_nlu_engine!(client);

    let intents = nlu_engine.get_intents(&input)?;
    let raw_pointer = CIntentClassifierResultArray::from(intents).into_raw_pointer();

    unsafe { *result = raw_pointer };

    Ok(())
}

fn run_parse_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intents_whitelist: *const CStringArray,
    intents_blacklist: *const CStringArray,
    result_json: *mut *const libc::c_char,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let nlu_engine = get_nlu_engine!(client);

    let opt_whitelist: Option<Vec<_>> = if !intents_whitelist.is_null() {
        Some(unsafe { convert_to_rust_vec(intents_whitelist)? })
    } else {
        None
    };
    let opt_blacklist: Option<Vec<_>> = if !intents_blacklist.is_null() {
        Some(unsafe { convert_to_rust_vec(intents_blacklist)? })
    } else {
        None
    };
    let results = nlu_engine.parse(&input, opt_whitelist, opt_blacklist)?;

    point_to_string(result_json, serde_json::to_string(&results)?)
}

fn run_get_slots_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    intent: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let intent = create_rust_string_from!(intent);
    let nlu_engine = get_nlu_engine!(client);

    let slots = nlu_engine.get_slots(&input, &intent)?;
    point_to_string(result_json, serde_json::to_string(&slots)?)
}

fn run_get_intents_into_json(
    client: *const CSnipsNluEngine,
    input: *const libc::c_char,
    result_json: *mut *const libc::c_char,
) -> Result<()> {
    let input = create_rust_string_from!(input);
    let nlu_engine = get_nlu_engine!(client);

    let intents = nlu_engine.get_intents(&input)?;

    point_to_string(result_json, serde_json::to_string(&intents)?)
}

fn get_model_version(version: *mut *const libc::c_char) -> Result<()> {
    point_to_string(version, snips_nlu_lib::MODEL_VERSION.to_string())
}

unsafe fn convert_to_rust_vec<'a>(c_array: *const CStringArray) -> Result<Vec<&'a str>> {
    let array = &*c_array;
    slice::from_raw_parts(array.data, array.size as usize)
        .into_iter()
        .map(|&ptr| Ok(CStr::from_ptr(ptr).to_str().map_err(failure::Error::from)?))
        .collect::<Result<Vec<_>>>()
}
