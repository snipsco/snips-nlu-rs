# coding=utf-8
from __future__ import unicode_literals

import unittest

from snips_nlu_rs import NLUEngine
from snips_nlu_rs.tests.utils import (SAMPLE_ASSISTANT_DIR,
                                      SAMPLE_ASSISTANT_ZIP)


class TestNLUEngineWrapper(unittest.TestCase):
    def test_should_load_from_path_and_parse(self):
        # Given
        engine = NLUEngine(data_path=SAMPLE_ASSISTANT_DIR)

        # When
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])

    def test_should_load_from_zip_and_parse(self):
        # Given
        engine = NLUEngine(data_zip=SAMPLE_ASSISTANT_ZIP)

        # Then
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])
