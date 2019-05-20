from _ctypes import Structure, POINTER, byref
from contextlib import contextmanager
from ctypes import cdll, c_char_p, c_int32, string_at
from pathlib import Path

dylib_dir = Path(__file__).parent / "dylib"
dylib_path = list(dylib_dir.glob("libsnips_nlu*"))[0]
lib = cdll.LoadLibrary(str(dylib_path))


@contextmanager
def string_pointer(ptr):
    try:
        yield ptr
    finally:
        lib.ffi_snips_nlu_engine_destroy_string(ptr)


class CStringArray(Structure):
    _fields_ = [
        ("data", POINTER(c_char_p)),
        ("size", c_int32)
    ]


def check_ffi_error(exit_code, error_context_msg):
    if exit_code != 0:
        with string_pointer(c_char_p()) as ptr:
            if lib.snips_nlu_engine_get_last_error(byref(ptr)) == 0:
                ffi_error_message = string_at(ptr).decode("utf8")
            else:
                ffi_error_message = "see stderr"
        raise ValueError("%s: %s" % (error_context_msg, ffi_error_message))
