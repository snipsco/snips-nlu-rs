extern crate libc;
extern crate snips_nlu_ffi;

use snips_nlu_ffi::{NLURESULT, Opaque};

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

export_c_symbol!(ffi_nlu_engine_create_from_dir, fn nlu_engine_create_from_dir(root_dir: *const libc::c_char, client: *mut *const Opaque) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_create_from_zip, fn nlu_engine_create_from_zip(zip: *const libc::c_uchar, zip_size: libc::c_uint, client: *mut *const Opaque) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_run_parse_into_json, fn nlu_engine_run_parse_into_json(client: *const Opaque, input: *const libc::c_char, result_json: *mut *const libc::c_char) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_get_last_error, fn nlu_engine_get_last_error(error: *mut *const libc::c_char) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_destroy_string, fn nlu_engine_destroy_string(string: *mut libc::c_char) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_destroy_client, fn nlu_engine_destroy_client(client: *mut Opaque) -> NLURESULT);
export_c_symbol!(ffi_nlu_engine_get_model_version, fn nlu_engine_get_model_version(version: *mut *const libc::c_char) -> NLURESULT);
