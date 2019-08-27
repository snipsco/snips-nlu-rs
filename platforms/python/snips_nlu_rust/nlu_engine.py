# coding=utf-8
from __future__ import (absolute_import, division, print_function,
                        unicode_literals)

import json
from builtins import object, str
from ctypes import byref, c_char_p, c_void_p, string_at, c_char, c_int
from pathlib import Path

from snips_nlu_rust.utils import (
    lib, string_pointer, CStringArray, check_ffi_error)


class NLUEngine(object):
    """Python wrapper of the Rust NLU engine

    This wrapper is similar, and can be used in conjunction with the NLU
    engine defined in the snips-nlu python library
    (https://github.com/snipsco/snips-nlu), except that it only provides
    APIs for inference and not for training.

    Examples:

        >>> import io
        >>> import json
        >>> from snips_nlu import SnipsNLUEngine as TrainingEngine
        >>> from snips_nlu_rust import NLUEngine as InferenceEngine
        >>> with io.open("/path/to/dataset.json") as f:
        ...     dataset = json.load(f)
        >>> engine = TrainingEngine().fit(dataset)
        >>> engine.persist("/path/to/nlu_engine")
        >>> inference_engine = InferenceEngine(
        ...     engine_dir="/path/to/nlu_engine")
        >>> inference_engine.parse("Turn on the lights in the kitchen")
    """

    def __init__(self, engine_dir=None, engine_bytes=None):
        self._engine = None

        if engine_dir is None and engine_bytes is None:
            raise ValueError("Please specify engine_dir or engine_bytes")

        if engine_dir is not None:
            engine_dir = Path(engine_dir)
            self._engine = c_void_p()
            exit_code = lib.ffi_snips_nlu_engine_create_from_dir(
                str(engine_dir).encode("utf8"), byref(self._engine))
            err_msg = "Something went wrong when creating the engine from " \
                      "directory"

        else:
            self._engine = c_void_p()
            bytearray_type = c_char * len(engine_bytes)
            exit_code = lib.ffi_snips_nlu_engine_create_from_zip(
                bytearray_type.from_buffer(engine_bytes), len(engine_bytes),
                byref(self._engine))
            err_msg = "Something went wrong when creating the engine from " \
                      "bytes"

        check_ffi_error(exit_code, err_msg)

    def parse(self, query, intents_whitelist=None, intents_blacklist=None,
              intents_alternatives=0, slots_alternatives=5):
        """Extracts intent and slots from an input query

        Args:
            query (str): input to process
            intents_whitelist (list of str, optional): if defined, this will
                restrict the scope of intent parsing to the provided intents
            intents_blacklist (list of str, optional): if defined, these
                intents will be excluded from the scope of intent parsing
            intents_alternatives (int, optional): number of alternative parsing
                results to include in the output.
            slots_alternatives (int, optional): number of alternative slot
                values to include along with each extracted slot.

        Returns:
            A python dict containing data about intent and slots. See
            https://snips-nlu.readthedocs.io/en/latest/tutorial.html#parsing
            for details about the format.
        """
        if intents_whitelist is not None:
            if not all(
                    isinstance(intent, str) for intent in intents_whitelist):
                raise TypeError("Expected 'intents_whitelist' to contain "
                                "objects of type 'str'")
            intents = [intent.encode("utf8") for intent in intents_whitelist]
            arr = CStringArray()
            arr.size = c_int(len(intents))
            arr.data = (c_char_p * len(intents))(*intents)
            intents_whitelist = byref(arr)
        if intents_blacklist is not None:
            if not all(
                    isinstance(intent, str) for intent in intents_blacklist):
                raise TypeError("Expected 'intents_blacklist' to contain "
                                "objects of type 'str'")
            intents = [intent.encode("utf8") for intent in intents_blacklist]
            arr = CStringArray()
            arr.size = c_int(len(intents))
            arr.data = (c_char_p * len(intents))(*intents)
            intents_blacklist = byref(arr)
        with string_pointer(c_char_p()) as ptr:
            exit_code = \
                lib.ffi_snips_nlu_engine_run_parse_with_alternatives_into_json(
                    self._engine, query.encode("utf8"), intents_whitelist,
                    intents_blacklist, intents_alternatives,
                    slots_alternatives, byref(ptr))
            msg = "Something went wrong when parsing query '%s'" % query
            check_ffi_error(exit_code, msg)
            result = string_at(ptr)

        return json.loads(result.decode("utf8"))

    def get_slots(self, query, intent, slots_alternatives=5):
        """Extracts slots from the input when the intent is known

        Args:
            query (str): input to process
            intent (str): intent which the input corresponds to
            slots_alternatives (int, optional): number of alternative slot
                values to include along with each extracted slot.

        Returns:
            A list of slots. See
            https://snips-nlu.readthedocs.io/en/latest/tutorial.html#parsing
            for details about the format.
        """
        with string_pointer(c_char_p()) as ptr:
            exit_code = lib.\
                ffi_snips_nlu_engine_run_get_slots_with_alternatives_into_json(
                    self._engine, query.encode("utf8"), intent.encode("utf8"),
                    slots_alternatives, byref(ptr))
            msg = "Something went wrong when extracting slots from query " \
                  "'%s' with intent '%s'" % (query, intent)
            check_ffi_error(exit_code, msg)
            result = string_at(ptr)

        return json.loads(result.decode("utf8"))

    def get_intents(self, query):
        """Returns all intents sorted by decreasing confidence scores

        Args:
            query (str): input to process

        Returns:
            A list of intents along with their probability. See
            https://snips-nlu.readthedocs.io/en/latest/tutorial.html#parsing
            for details about the format.
        """
        with string_pointer(c_char_p()) as ptr:
            exit_code = lib.ffi_snips_nlu_engine_run_get_intents_into_json(
                self._engine, query.encode("utf8"), byref(ptr))
            msg = "Something went wrong when extracting intents from query " \
                  "'%s'" % query
            check_ffi_error(exit_code, msg)
            result = string_at(ptr)
        return json.loads(result.decode("utf8"))

    def __del__(self):
        if self._engine is not None and lib is not None:
            lib.ffi_snips_nlu_engine_destroy_client(self._engine)
