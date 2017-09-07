# coding=utf-8
from __future__ import unicode_literals

import unittest

from snips_nlu_rust import NLUEngine
from snips_nlu_rust.tests.utils import (SAMPLE_ASSISTANT_DIR,
                                        SAMPLE_ASSISTANT_ZIP)


class TestNLUEngineWrapper(unittest.TestCase):
    def test_should_load_with_path_and_parse(self):
        # Given
        engine = NLUEngine('en', data_path=SAMPLE_ASSISTANT_DIR)

        # When
        result = engine.parse("Make me two cups of coffee please")

        # Then
        expected_dict = {
            'input': 'Make me two cups of coffee please',
            'intent': {'intentName': 'MakeCoffee', 'probability': 0.7035172},
            'slots': [
                {
                    'entity': 'snips/number',
                    'range': {'end': 11, 'start': 8},
                    'rawValue': 'two',
                    'slotName': 'number_of_cups',
                    'value': {'kind': 'Number', 'value': 2.0}
                }
            ]
        }

        self.assertDictEqual(expected_dict, result)

    def test_should_load_with_zip_and_parse(self):
        # Given
        engine = NLUEngine('en', data_zip=SAMPLE_ASSISTANT_ZIP)

        # When
        result = engine.parse("Make me two cups of coffee please")

        # Then
        expected_dict = {
            'input': 'Make me two cups of coffee please',
            'intent': {'intentName': 'MakeCoffee', 'probability': 0.7035172},
            'slots': [
                {
                    'entity': 'snips/number',
                    'range': {'end': 11, 'start': 8},
                    'rawValue': 'two',
                    'slotName': 'number_of_cups',
                    'value': {'kind': 'Number', 'value': 2.0}
                }
            ]
        }

        self.assertDictEqual(expected_dict, result)
