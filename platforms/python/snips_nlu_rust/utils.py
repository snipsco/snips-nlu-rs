from _ctypes import Structure, POINTER
from contextlib import contextmanager
from ctypes import cdll, c_char_p, c_int32
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
