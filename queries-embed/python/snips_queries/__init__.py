from ctypes import *
import os

#lib = cdll.LoadLibrary('../../target/debug/libqueries_embed.so')
lib = cdll.LoadLibrary(os.path.join(os.path.dirname(__file__), '../queries_embed.so'))

class SnipsQueries(object):
    def __init__(self, data_path, intent_types):
        intent_types_arg = (c_char_p * len(intent_types))()
        intent_types_arg[:] = list(map(lambda s: s.encode("utf-8"), intent_types))
        self.obj = pointer(c_void_p()) 
        self.ok = lib.intent_parser_create(data_path.encode('utf-8'), byref(self.obj), len(intent_types), intent_types_arg )
        if self.ok != 1:
            raise RuntimeError("something wrong happened while creating the client, see stderr")

    def __del__(self):
        lib.intent_parser_destroy_client(self.obj)

    def run_intent_classification(self, input, probability_threshold):
        pointer = c_char_p()
        lib.intent_parser_run_intent_classification(self.obj, input.encode("utf-8"), c_float(probability_threshold), byref(pointer))
        result = string_at(pointer)
        lib.intent_parser_destroy_string(pointer)
        return result

    def run_tokens_classification(self, input, intent_name):
        pointer = c_char_p()
        lib.intent_parser_run_tokens_classification(self.obj, input.encode("utf-8"), intent_name.encode("utf-8"), byref(pointer))
        result = string_at(pointer)
        lib.intent_parser_destroy_string(pointer)
        return result

