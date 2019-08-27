from __future__ import unicode_literals

import io
import os

TEST_DATA_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                              "..", "..", "..", "..", "data", "tests")

GAME_ENGINE_DIR = os.path.join(TEST_DATA_PATH, "models", "nlu_engine_game")
BEVERAGE_ENGINE_DIR = os.path.join(TEST_DATA_PATH, "models",
                                   "nlu_engine_beverage")
BEVERAGE_ENGINE_ZIP_PATH = os.path.join(TEST_DATA_PATH, "models",
                                        "nlu_engine_beverage.zip")

with io.open(BEVERAGE_ENGINE_ZIP_PATH, mode='rb') as f:
    BEVERAGE_ENGINE_ZIP_BYTES = bytearray(f.read())
