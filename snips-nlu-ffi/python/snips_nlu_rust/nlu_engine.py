# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

import json
import os
from builtins import object
from ctypes import c_char, c_char_p, c_void_p, string_at, pointer, byref

from snips_nlu_rust.utils import lib, string_pointer


class NLUEngine(object):
    def __init__(self, data_path=None, data_zip=None):
        exit_code = 1
        self._engine = None

        if data_path is None and data_zip is None:
            raise ValueError("Please specify data_path or data_zip")

        if data_path is not None:
            self._engine = pointer(c_void_p())
            if os.path.isdir(data_path):
                exit_code = lib.ffi_snips_nlu_engine_create_from_dir(
                    data_path.encode("utf-8"), byref(self._engine))
            else:
                exit_code = lib.ffi_snips_nlu_engine_create_from_file(
                    data_path.encode("utf-8"), byref(self._engine))

        if data_zip is not None:
            self._engine = pointer(c_void_p())
            bytearray_type = c_char * len(data_zip)
            exit_code = lib.ffi_snips_nlu_engine_create_from_zip(
                bytearray_type.from_buffer(data_zip), len(data_zip),
                byref(self._engine))

        if exit_code:
            raise ImportError('Something wrong happened while creating the '
                              'intent parser. See stderr.')


    def __del__(self):
        if self._engine is not None:
            lib.ffi_snips_nlu_engine_destroy_client(self._engine)


    def parse(self, query):
        with string_pointer(c_char_p()) as ptr:
            lib.ffi_snips_nlu_engine_run_parse_into_json(
                self._engine, query.encode("utf-8"), byref(ptr))
            result = string_at(ptr)

        return json.loads(result.decode("utf-8"))
