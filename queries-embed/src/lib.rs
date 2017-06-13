extern crate libc;
extern crate queries_core;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

extern crate serde_json;

use std::ffi::{CStr, CString};
use std::sync::Mutex;
use std::slice;
use std::io::Cursor;

use libc::c_char;

lazy_static! {
    static ref LAST_ERROR:std::sync::Mutex<String> = std::sync::Mutex::new("".to_string());
}

mod errors {
    error_chain! {
          foreign_links {
                Core(::queries_core::Error);
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
pub struct Opaque(std::sync::Mutex<queries_core::SnipsNLUEngine>);

#[repr(C)]
pub enum QUERIESRESULT {
    KO = 0,
    OK = 1,
}

macro_rules! wrap {
    ($e:expr) => { match $e {
        Ok(_) => { return QUERIESRESULT::OK; }
        Err(e) => {
            use std::io::Write;
            use error_chain::ChainedError;
            let stderr = &mut ::std::io::stderr();
            let errmsg = "Error writing to stderr";
            let msg = e.display().to_string();
            writeln!(stderr, "{}", msg).expect(errmsg);
            match LAST_ERROR.lock() {
                Ok(mut guard) => *guard = msg,
                Err(_) => () /* curl up and cry */
            }
            return QUERIESRESULT::KO;
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

macro_rules! get_str_vec {
    ($size:ident, $pointer:ident) => {{
        let mut strings: Vec<&str> = Vec::new();
        for &s in unsafe { slice::from_raw_parts($pointer, $size as usize) } {
            strings.push(get_str!(s))
        }
        strings
    }}
}

#[repr(C)]
pub struct CIntentParserResult {
    pub input: *const libc::c_char,
    pub intent: Option<Box<CIntentClassifierResult>>,
    pub slots: Option<Box<CSlotList>>,
}

impl CIntentParserResult {
    fn from(input: queries_core::IntentParserResult) -> Result<Self> {
        Ok(CIntentParserResult {
            input: CString::new(input.input)?.into_raw(),
            intent: if let Some(intent) = input.intent {
                Some(Box::new(CIntentClassifierResult::from(intent)?))
            } else { None },
            slots: if let Some(slots) = input.slots {
                Some(Box::new(CSlotList::from(slots)?))
            } else { None },
        })
    }
}

impl Drop for CIntentParserResult {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.input as *mut libc::c_char) };
    }
}

#[repr(C)]
pub struct CIntentClassifierResult {
    pub intent_name: *const libc::c_char,
    pub probability: libc::c_float,
}

impl CIntentClassifierResult {
    fn from(input: queries_core::IntentClassifierResult) -> Result<Self> {
        Ok(CIntentClassifierResult {
            probability: input.probability,
            intent_name: CString::new(input.intent_name)?.into_raw(),
        })
    }
}

impl Drop for CIntentClassifierResult {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.intent_name as *mut libc::c_char) };
    }
}

#[repr(C)]
pub struct CSlotList {
    pub slots: Box<[CSlot]>,
    pub size: libc::c_int,
}

impl CSlotList {
    fn from(input: Vec<queries_core::Slot>) -> Result<Self> {
        Ok(CSlotList {
            size: input.len() as libc::c_int,
            slots: input.into_iter().map(|s| CSlot::from(s)).collect::<Result<Vec<CSlot>>>()?.into_boxed_slice()
        })
    }
}

#[repr(C)]
pub struct CSlot {
    pub value: *const libc::c_char,
    pub range_start: libc::c_int,
    pub range_end: libc::c_int,
    pub entity: *const libc::c_char,
    pub slot_name: *const libc::c_char
}

impl CSlot {
    fn from(input: queries_core::Slot) -> Result<Self> {
        Ok(CSlot {
            value: CString::new(input.raw_value)?.into_raw(),
            range_start: input.range.start as libc::c_int,
            range_end: input.range.end as libc::c_int,
            entity: CString::new(input.entity)?.into_raw(),
            slot_name: CString::new(input.slot_name)?.into_raw()
        })
    }
}

impl Drop for CSlot {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.value as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.entity as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.slot_name as *mut libc::c_char) };
    }
}


#[no_mangle]
pub extern "C" fn nlu_engine_create_from_dir(root_dir: *const c_char,
                                             client: *mut *mut Opaque)
                                             -> QUERIESRESULT {
    wrap!(create_from_dir(root_dir, client));
}

#[no_mangle]
pub extern "C" fn nlu_engine_create_from_binary(binary: *const libc::c_uchar,
                                                   binary_size: libc::c_uint,
                                                   client: *mut *mut Opaque)
                                                   -> QUERIESRESULT {
    wrap!(create_from_binary(binary, binary_size, client));
}

#[no_mangle]
pub extern "C" fn nlu_engine_run_parse(client: *mut Opaque,
                                       input: *const c_char,
                                       result: *mut *const CIntentParserResult)
                                       -> QUERIESRESULT {
    wrap!(run_parse(client, input, result))
}

#[no_mangle]
pub extern "C" fn nlu_engine_run_parse_into_json(client: *mut Opaque,
                                                 input: *const c_char,
                                                 result_json: *mut *mut c_char)
                                                 -> QUERIESRESULT {
    wrap!(run_parse_into_json(client, input, result_json))
}

#[no_mangle]
pub extern "C" fn nlu_engine_get_last_error(error: *mut *mut c_char) -> QUERIESRESULT {
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

fn create_from_dir(root_dir: *const libc::c_char, client: *mut *mut Opaque) -> Result<()> {
    let root_dir = get_str!(root_dir);

    let assistant_config = queries_core::FileBasedConfiguration::new(root_dir)?;
    let intent_parser = queries_core::SnipsNLUEngine::new(assistant_config)?;

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };

    Ok(())
}

fn create_from_binary(binary: *const libc::c_uchar,
                      binary_size: libc::c_uint,
                      client: *mut *mut Opaque)
                      -> Result<()> {
    let slice = unsafe { slice::from_raw_parts(binary, binary_size as usize) };
    let reader = Cursor::new(slice.to_owned());

    let assistant_config = queries_core::BinaryBasedConfiguration::new(reader)?;
    let intent_parser = queries_core::SnipsNLUEngine::new(assistant_config)?;

    unsafe { *client = Box::into_raw(Box::new(Opaque(Mutex::new(intent_parser)))) };

    Ok(())
}

fn run_parse(client: *mut Opaque,
             input: *const c_char,
             result: *mut *const CIntentParserResult)
             -> Result<()> {
    let input = get_str!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(input, None)?;
    let b = Box::new(CIntentParserResult::from(results)?);

    unsafe { *result = Box::into_raw(b) as *const CIntentParserResult }
    Ok(())
}


fn run_parse_into_json(client: *mut Opaque,
                       input: *const c_char,
                       result_json: *mut *mut c_char)
                       -> Result<()> {
    let input = get_str!(input);
    let intent_parser = get_intent_parser!(client);

    let results = intent_parser.parse(input, None)?;

    point_to_string(result_json, serde_json::to_string(&results)?)
}

fn get_last_error(error: *mut *mut c_char) -> Result<()> {
    point_to_string(error, LAST_ERROR.lock()?.clone())
}


fn point_to_string(pointer: *mut *mut libc::c_char, string: String) -> Result<()> {
    let cs = CString::new(string.as_bytes())?;
    unsafe { *pointer = cs.into_raw() }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_size() {
        println!("{}", std::mem::size_of::<CIntentParserResult>());
        println!("{}", std::mem::size_of::<*const CIntentClassifierResult>());
        println!("{}", std::mem::size_of::<Option<*const CSlotList>>());
    }
}
