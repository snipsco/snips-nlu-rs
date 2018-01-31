# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

import json
import os
from builtins import object, bytes
from ctypes import c_char, c_char_p, c_void_p, string_at, pointer, byref, cdll
from glob import glob

dylib_path = glob(
    os.path.join(os.path.dirname(__file__), "dylib", "libsnips_nlu*"))[0]
lib = cdll.LoadLibrary(dylib_path)


class NLUEngine(object):
    def __init__(self, data_path=None, data_zip=None):
        exit_code = 1

        if data_path is None and data_zip is None:
            raise ValueError("Please specify data_path or data_zip")

        if data_path is not None:
            self.data_path = data_path
            self._engine = pointer(c_void_p())
            exit_code = lib.nlu_engine_create_from_dir(
                data_path.encode("utf-8"), byref(self._engine))

        if data_zip is not None:
            self._engine = pointer(c_void_p())
            bytearray_type = c_char * len(data_zip)
            exit_code = lib.nlu_engine_create_from_zip(
                bytearray_type.from_buffer(data_zip), len(data_zip),
                byref(self._engine))

        if exit_code != 1:
            raise ImportError('Something wrong happened while creating the '
                              'intent parser. See stderr.')

    def __del__(self):
        lib.nlu_engine_destroy_client(self._engine)

    def parse(self, query):
        pointer = c_char_p()
        lib.nlu_engine_run_parse_into_json(
            self._engine,
            query.encode("utf-8"),
            byref(pointer))
        result = string_at(pointer)
        lib.nlu_engine_destroy_string(pointer)

        return json.loads(bytes(result).decode())
