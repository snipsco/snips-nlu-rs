extern crate ffi_utils;
extern crate libc;
extern crate snips_nlu_ffi;

use ffi_utils::SNIPS_RESULT;
use snips_nlu_ffi::CSnipsNluEngine;

#[doc(hidden)]
#[macro_export]
macro_rules! export_c_symbol {
    ($alias:ident, fn $name:ident($( $arg:ident : $type:ty ),*) -> $ret:ty) => {
        #[no_mangle]
        pub extern "C" fn $alias($( $arg : $type),*) -> $ret {
            ::snips_nlu_ffi::$name($( $arg ),*)
        }
    };
    ($alias:ident, fn $name:ident($( $arg:ident : $type:ty ),*)) => {
        export_c_symbol!($alias, fn $name($( $arg : $type),*) -> ());
    }
}

export_c_symbol!(ffi_snips_nlu_engine_create_from_dir, fn snips_nlu_engine_create_from_dir(root_dir: *const libc::c_char, client: *mut *const CSnipsNluEngine) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_create_from_zip, fn snips_nlu_engine_create_from_zip(zip: *const libc::c_uchar, zip_size: libc::c_uint, client: *mut *const CSnipsNluEngine) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_run_parse_into_json, fn snips_nlu_engine_run_parse_into_json(client: *const CSnipsNluEngine, input: *const libc::c_char, result_json: *mut *const libc::c_char) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_get_last_error, fn snips_nlu_engine_get_last_error(error: *mut *const libc::c_char) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_destroy_string, fn snips_nlu_engine_destroy_string(string: *mut libc::c_char) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_destroy_client, fn snips_nlu_engine_destroy_client(client: *mut CSnipsNluEngine) -> SNIPS_RESULT);
export_c_symbol!(ffi_snips_nlu_engine_get_model_version, fn snips_nlu_engine_get_model_version(version: *mut *const libc::c_char) -> SNIPS_RESULT);
