from __future__ import unicode_literals

import io
import os

TEST_DATA_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                              "..", "..", "..", "..", "data", "tests")

SAMPLE_ASSISTANT_DIR = os.path.join(TEST_DATA_PATH, "configurations")
SAMPLE_ASSISTANT_FILE = os.path.join(TEST_DATA_PATH, "configurations",
                                     "trained_assistant.json")
SAMPLE_ASSISTANT_ZIP_PATH = os.path.join(TEST_DATA_PATH, "zip_files",
                                         "sample_config.zip")

with io.open(SAMPLE_ASSISTANT_ZIP_PATH, mode='rb') as f:
    SAMPLE_ASSISTANT_ZIP = bytearray(f.read())
