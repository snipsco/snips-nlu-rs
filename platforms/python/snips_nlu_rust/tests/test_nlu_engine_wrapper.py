# coding=utf-8
from __future__ import unicode_literals

import unittest

from snips_nlu_rust import NLUEngine
from snips_nlu_rust.tests.utils import (
    BEVERAGE_ENGINE_DIR, BEVERAGE_ENGINE_ZIP_BYTES, GAME_ENGINE_DIR)


class TestNLUEngineWrapper(unittest.TestCase):
    def test_should_load_from_dir_and_parse(self):
        # Given
        engine = NLUEngine(engine_dir=BEVERAGE_ENGINE_DIR)

        # When
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])

    def test_load_from_dir_should_fail_with_invalid_path(self):
        with self.assertRaises(ValueError) as cm:
            NLUEngine(engine_dir="/tmp/invalid/path/to/engine")

        self.assertTrue("No such file or directory" in str(cm.exception))

    def test_should_load_from_zip_and_parse(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please")

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])
        self.assertEqual(0, len(res["alternatives"]))

    def test_load_from_zip_should_fail_with_invalid_data(self):
        with self.assertRaises(ValueError) as cm:
            NLUEngine(engine_bytes=bytearray())

        self.assertTrue("Invalid Zip archive" in str(cm.exception))

    def test_should_parse_with_whitelist(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please",
                           intents_whitelist=["MakeTea"])

        # Then
        self.assertEqual("MakeTea", res["intent"]["intentName"])

    def test_should_parse_with_blacklist(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please",
                           intents_blacklist=["MakeCoffee"])

        # Then
        self.assertEqual("MakeTea", res["intent"]["intentName"])

    def test_should_parse_with_intents_alternatives(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        res = engine.parse("Make me two cups of coffee please",
                           intents_alternatives=1)

        # Then
        self.assertEqual("MakeCoffee", res["intent"]["intentName"])
        self.assertEqual(1, len(res["alternatives"]))
        self.assertEqual("MakeTea",
                         res["alternatives"][0]["intent"]["intentName"])

    def test_should_parse_with_slots_alternatives(self):
        # Given
        engine = NLUEngine(engine_dir=GAME_ENGINE_DIR)

        # When
        result = engine.parse("I want to play to invader",
                              slots_alternatives=2)
        result["intent"]["confidenceScore"] = 0.8

        # Then
        expected_slots = [
            {
                "rawValue": "invader",
                "value": {
                    "kind": "Custom",
                    "value": "Invader Attack 3"
                },
                "alternatives": [
                    {
                        "kind": "Custom",
                        "value": "Invader War Demo"
                    },
                    {
                        "kind": "Custom",
                        "value": "Space Invader Limited Edition"
                    },
                ],
                "range": {
                    "start": 18,
                    "end": 25
                },
                "slotName": "game",
                "entity": "game",
            }
        ]
        expected_result = {
            "input": "I want to play to invader",
            "intent": {
                "intentName": "PlayGame",
                "confidenceScore": 0.8
            },
            "slots": expected_slots,
            "alternatives": [],
        }
        self.assertEqual(expected_result, result)

    def test_should_get_slots(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        slots = engine.get_slots("Make me two cups of coffee please",
                                 intent="MakeCoffee")

        # Then
        expected_slots = [
            {
                "entity": "snips/number",
                "range": {"end": 11, "start": 8},
                "rawValue": "two",
                "slotName": "number_of_cups",
                "value": {"kind": "Number", "value": 2.0},
                "alternatives": []
            }
        ]
        self.assertEqual(expected_slots, slots)

    def test_should_get_slots_with_alternatives(self):
        # Given
        engine = NLUEngine(engine_dir=GAME_ENGINE_DIR)

        # When
        slots = engine.get_slots("I want to play to invader",
                                 intent="PlayGame", slots_alternatives=2)

        # Then
        expected_slots = [
            {
                "rawValue": "invader",
                "value": {
                    "kind": "Custom",
                    "value": "Invader Attack 3"
                },
                "alternatives": [
                    {
                        "kind": "Custom",
                        "value": "Invader War Demo"
                    },
                    {
                        "kind": "Custom",
                        "value": "Space Invader Limited Edition"
                    },
                ],
                "range": {
                    "start": 18,
                    "end": 25
                },
                "slotName": "game",
                "entity": "game",
            }
        ]
        self.assertEqual(expected_slots, slots)

    def test_get_slots_should_fail_with_unknown_intent(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        with self.assertRaises(ValueError) as cm:
            engine.get_slots(
                "Make me two cups of coffee please", intent="my_intent")
        self.assertTrue("Unknown intent" in str(cm.exception))

    def test_should_get_intents(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # Then
        intents_results = engine.get_intents(
            "Make me two cups of coffee please")
        intents = [res["intentName"] for res in intents_results]

        # Then
        expected_intents = ["MakeCoffee", "MakeTea", None]
        self.assertEqual(expected_intents, intents)

    def test_engine_should_destroy_itself(self):
        # Given
        engine = NLUEngine(engine_bytes=BEVERAGE_ENGINE_ZIP_BYTES)

        # When / Then
        del engine
