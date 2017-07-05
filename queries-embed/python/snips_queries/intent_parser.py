# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

import json
import os
from ctypes import c_char, c_char_p, c_void_p, string_at, pointer, byref, cdll

lib = cdll.LoadLibrary(os.path.join(os.path.dirname(__file__), "../libsnips_queries.so"))
# lib = cdll.LoadLibrary("../../target/debug/libsnips_queries.so") # use for dev
# lib = cdll.LoadLibrary("../../target/debug/libsnips_queries.dylib") # use for dev

class IntentParser(object):
    def __init__(self, language, data_path=None, data_zip=None):
        self.language = language
        exit_code = 1

        if data_path is None and data_zip is None:
            raise ValueError("Please specify data_path or data_zip")

        if data_path is not None:
            self.data_path = data_path
            self._parser = pointer(c_void_p())
            exit_code = lib.nlu_engine_create_from_dir(
                data_path.encode("utf-8"), byref(self._parser))

        if data_zip is not None:
            self._parser = pointer(c_void_p())
            bytearray_type = c_char * len(data_zip)
            exit_code = lib.nlu_engine_create_from_zip(
                bytearray_type.from_buffer(data_zip), len(data_zip), byref(self._parser))

        if exit_code != 1:
            raise ImportError('Something wrong happened while creating the '
                              'intent parser. See stderr.')

    def __del__(self):
        lib.intent_parser_destroy_client(self._parser)

    def parse(self, query):
        pointer = c_char_p()
        lib.nlu_engine_run_parse_into_json(
            self._parser,
            query.encode("utf-8"),
            byref(pointer))
        result = string_at(pointer)
        lib.intent_parser_destroy_string(pointer)

        return json.loads(result)
