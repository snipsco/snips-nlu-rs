# coding=utf-8
from __future__ import (absolute_import, division, print_function,
                        unicode_literals)

import json
from builtins import object, str
from ctypes import byref, c_char_p, c_void_p, pointer, string_at, c_char, c_int
from pathlib import Path

from snips_nlu_rust.utils import lib, string_pointer, CStringArray


class NLUEngine(object):
    def __init__(self, engine_dir=None, engine_bytes=None):
        exit_code = 1
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
                str(engine_dir).encode("utf8"), byref(self._engine))
        elif engine_bytes is not None:
            self._engine = pointer(c_void_p())
            bytearray_type = c_char * len(engine_bytes)
            exit_code = lib.ffi_snips_nlu_engine_create_from_zip(
                bytearray_type.from_buffer(engine_bytes), len(engine_bytes),
                byref(self._engine))

        if exit_code:
            raise ImportError('Something wrong happened while creating the '
                              'intent parser. See stderr.')

    def parse(self, query, intents_whitelist=None, intents_blacklist=None):
        if intents_whitelist is not None:
            if not all(isinstance(intent, str) for intent in intents_whitelist):
                raise TypeError(
                    "Expected 'intents_whitelist' to contain objects of type 'str'")
            intents = [intent.encode("utf8") for intent in intents_whitelist]
            arr = CStringArray()
            arr.size = c_int(len(intents))
            arr.data = (c_char_p * len(intents))(*intents)
            intents_whitelist = byref(arr)
        if intents_blacklist is not None:
            if not all(isinstance(intent, str) for intent in intents_blacklist):
                raise TypeError(
                    "Expected 'intents_blacklist' to contain objects of type 'str'")
            intents = [intent.encode("utf8") for intent in intents_blacklist]
            arr = CStringArray()
            arr.size = c_int(len(intents))
            arr.data = (c_char_p * len(intents))(*intents)
            intents_blacklist = byref(arr)
        with string_pointer(c_char_p()) as ptr:
            lib.ffi_snips_nlu_engine_run_parse_into_json(
                self._engine, query.encode("utf8"), intents_whitelist, intents_blacklist,
                byref(ptr))
            result = string_at(ptr)

        return json.loads(result.decode("utf8"))

    def get_slots(self, query, intent):
        with string_pointer(c_char_p()) as ptr:
            lib.ffi_snips_nlu_engine_run_get_slots_into_json(
                self._engine, query.encode("utf8"), intent.encode("utf8"), byref(ptr))
            result = string_at(ptr)
        return json.loads(result.decode("utf8"))

    def get_intents(self, query):
        with string_pointer(c_char_p()) as ptr:
            lib.ffi_snips_nlu_engine_run_get_intents_into_json(
                self._engine, query.encode("utf8"), byref(ptr))
            result = string_at(ptr)
        return json.loads(result.decode("utf8"))

    def __del__(self):
        if self._engine is not None and lib is not None:
            lib.ffi_snips_nlu_engine_destroy_client(self._engine)
