# coding=utf-8
from __future__ import (absolute_import, division, print_function,
                        unicode_literals)

import json
from builtins import object, str
from ctypes import byref, c_char_p, c_void_p, pointer, string_at, c_char
from pathlib import Path

from snips_nlu_rust.utils import lib, string_pointer


class NLUEngine(object):
    def __init__(self, engine_dir=None, engine_bytes=None):
        exit_code=1
        self._engine = None

        if engine_dir is None and engine_bytes is None:
            raise ValueError("Please specify engine_dir or engine_bytes")

        if engine_dir is not None:
            engine_dir = Path(engine_dir)
            if not engine_dir.is_dir():
                raise OSError("NLU engine directory not found: %s"
                              % str(engine_dir))
            self._engine = pointer(c_void_p())
            exit_code = lib.ffi_snips_nlu_engine_create_from_dir(
                str(engine_dir).encode("utf-8"), byref(self._engine))
        elif engine_bytes is not None:
            self._engine = pointer(c_void_p())
            bytearray_type = c_char * len(engine_bytes)
            exit_code = lib.ffi_snips_nlu_engine_create_from_zip(
                bytearray_type.from_buffer(engine_bytes), len(engine_bytes),
                byref(self._engine))

        if exit_code:
            raise ImportError('Something wrong happened while creating the '
                              'intent parser. See stderr.')

    def __del__(self):
        if self._engine is not None and lib is not None:
            lib.ffi_snips_nlu_engine_destroy_client(self._engine)

    def parse(self, query):
        with string_pointer(c_char_p()) as ptr:
            lib.ffi_snips_nlu_engine_run_parse_into_json(
                self._engine, query.encode("utf-8"), byref(ptr))
            result = string_at(ptr)

        return json.loads(result.decode("utf-8"))
