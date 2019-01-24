from __future__ import unicode_literals

import io
import os

TEST_DATA_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                              "..", "..", "..", "..", "data", "tests")

SAMPLE_ENGINE_DIR = os.path.join(TEST_DATA_PATH, "models", "nlu_engine")
SAMPLE_ENGINE_ZIP_PATH = os.path.join(TEST_DATA_PATH, "models",
                                      "nlu_engine.zip")

with io.open(SAMPLE_ENGINE_ZIP_PATH, mode='rb') as f:
    SAMPLE_ENGINE_ZIP_BYTES = bytearray(f.read())
