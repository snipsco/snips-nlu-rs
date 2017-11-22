# coding=utf-8
from __future__ import unicode_literals

import argparse
import io
import json
import os
import shutil
import zipfile
from pprint import pprint
from tempfile import mkdtemp

from nlu_engine import NLUEngine

TRAINED_ENGINE_FILENAME = "trained_assistant.json"


def debug_inference(engine_path):
    with io.open(os.path.abspath(engine_path), "r", encoding="utf8") as f:
        engine_dict = json.load(f)

    language = engine_dict["language"]
    engine_dir = mkdtemp()
    try:
        trained_engine_path = os.path.join(engine_dir,
                                           TRAINED_ENGINE_FILENAME)
        archive_path = os.path.join(engine_dir, 'assistant.zip')

        with io.open(trained_engine_path, mode='w', encoding='utf8') as f:
            f.write(json.dumps(engine_dict).decode())
        with zipfile.ZipFile(archive_path, 'w') as zf:
            zf.write(trained_engine_path, arcname=TRAINED_ENGINE_FILENAME)
        with io.open(archive_path, mode='rb') as f:
            data_zip = bytearray(f.read())
    except Exception as e:
        raise Exception("Error while creating engine from zip archive: %s"
                        % e.message)
    finally:
        shutil.rmtree(engine_dir)
    engine = NLUEngine(language, data_zip=data_zip)

    while True:
        query = raw_input("Enter a query (type 'q' to quit): ").strip()
        if isinstance(query, str):
            query = query.decode("utf8")
        if query == "q":
            break
        print(json.dumps(engine.parse(query), indent=4))


def main_debug():
    parser = argparse.ArgumentParser(description="Debug snippets")
    parser.add_argument("engine_path", type=unicode,
                        help="Path to the trained assistant")
    args = vars(parser.parse_args())
    debug_inference(**args)


if __name__ == '__main__':
    main_debug()
