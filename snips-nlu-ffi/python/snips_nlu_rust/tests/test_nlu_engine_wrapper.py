# coding=utf-8
from __future__ import unicode_literals

import unittest

from snips_nlu_rust import NLUEngine
from snips_nlu_rust.tests.utils import (
    SAMPLE_ENGINE_DIR, SAMPLE_ENGINE_ZIP_BYTES)


class TestNLUEngineWrapper(unittest.TestCase):
    def test_should_load_from_dir_and_parse(self):
        # Given
        engine = NLUEngine(engine_dir=SAMPLE_ENGINE_DIR)

        # When
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])

    def test_should_load_from_zip_and_parse(self):
        # Given
        engine = NLUEngine(engine_bytes=SAMPLE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])
