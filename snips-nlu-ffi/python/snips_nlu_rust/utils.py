import os
from contextlib import contextmanager
from ctypes import cdll
from glob import glob

dylib_dir = os.path.join(os.path.dirname(__file__), "dylib")
dylib_path = glob(os.path.join(dylib_dir, "libsnips_nlu*"))[0]
lib = cdll.LoadLibrary(dylib_path)


@contextmanager
def string_pointer(ptr):
    try:
        yield ptr
    finally:
        lib.ffi_snips_nlu_engine_destroy_string(ptr)
