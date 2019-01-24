from contextlib import contextmanager
from ctypes import cdll
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
