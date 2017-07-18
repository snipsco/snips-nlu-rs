from __future__ import unicode_literals

import io
import os

TEST_PATH = os.path.dirname(os.path.abspath(__file__))

SAMPLE_ASSISTANT_DIR = os.path.join(TEST_PATH, "resources", "sample_assistant")
SAMPLE_ASSISTANT_ZIP_PATH = os.path.join(TEST_PATH, "resources",
                                         "sample_assistant.zip")

with io.open(SAMPLE_ASSISTANT_ZIP_PATH, mode='rb') as f:
    SAMPLE_ASSISTANT_ZIP = bytearray(f.read())
