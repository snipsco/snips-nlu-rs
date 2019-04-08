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

    def test_should_parse_with_whitelist(self):
        # Given
        engine = NLUEngine(engine_bytes=SAMPLE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please", intents_whitelist=["MakeTea"])

        # Then
        self.assertEqual("MakeTea", res["intent"]["intentName"])

    def test_should_parse_with_blacklist(self):
        # Given
        engine = NLUEngine(engine_bytes=SAMPLE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please", intents_blacklist=["MakeCoffee"])

        # Then
        self.assertEqual("MakeTea", res["intent"]["intentName"])

    def test_should_get_slots(self):
        # Given
        engine = NLUEngine(engine_bytes=SAMPLE_ENGINE_ZIP_BYTES)

        # Then
        slots = engine.get_slots("Make me two cups of coffee please", intent="MakeCoffee")

        # Then
        expected_slots = [
            {
                "entity": "snips/number",
                "range": {"end": 11, "start": 8},
                "rawValue": "two",
                "slotName": "number_of_cups",
                "value": {"kind": "Number", "value": 2.0}
            }
        ]
        self.assertEqual(expected_slots, slots)

    def test_should_get_intents(self):
        # Given
        engine = NLUEngine(engine_bytes=SAMPLE_ENGINE_ZIP_BYTES)

        # Then
        intents_results = engine.get_intents("Make me two cups of coffee please")
        intents = [res["intentName"] for res in intents_results]

        # Then
        expected_intents = ["MakeCoffee", "MakeTea", None]
        self.assertEqual(expected_intents, intents)
