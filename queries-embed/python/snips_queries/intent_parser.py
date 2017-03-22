# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

from ctypes import c_char_p, c_void_p, c_float, string_at, pointer, byref, cdll
import os
import json

lib = cdll.LoadLibrary(os.path.join(os.path.dirname(__file__), "../queries_embed.so"))

class IntentParser(object):

    def __init__(self, data_path):
        self.data_path = data_path
        self._parser = pointer(c_void_p())
        exit_code = lib.intent_parser_create(data_path, byref(self._parser))

        if exit_code != 1:
            raise ImportError('Something wrong happened while creating the '
                'intent parser. See stderr.')

    def __del__(self):
        lib.intent_parser_destroy_client(self._parser)

    def get_intent(self, query, threshold=0.):
        pointer = c_char_p()
        lib.intent_parser_run_intent_classification(self._parser, query,
            c_float(threshold), byref(pointer))
        result = string_at(pointer)
        lib.intent_parser_destroy_string(pointer)

        return json.loads(result)

    def get_entities(self, query, intent):
        pointer = c_char_p()
        lib.intent_parser_run_tokens_classification(self._parser, query,
            intent, byref(pointer))
        result = string_at(pointer)
        lib.intent_parser_destroy_string(pointer)

        return json.loads(result)
