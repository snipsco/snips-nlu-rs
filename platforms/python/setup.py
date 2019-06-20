from __future__ import print_function

import io
import os
import sys

from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

packages = [p for p in find_packages() if "tests" not in p]

PACKAGE_NAME = "snips_nlu_rust"
ROOT_PATH = os.path.dirname(os.path.abspath(__file__))
PACKAGE_PATH = os.path.join(ROOT_PATH, PACKAGE_NAME)
VERSION = "__version__"
README = os.path.join(ROOT_PATH, "README.rst")

RUST_EXTENSION_NAME = 'snips_nlu_rust.dylib.libsnips_nlu_rs'
CARGO_ROOT_PATH = os.path.join(ROOT_PATH, 'ffi')
CARGO_FILE_PATH = os.path.join(CARGO_ROOT_PATH, 'Cargo.toml')
CARGO_TARGET_DIR = os.path.join(CARGO_ROOT_PATH, 'target')
os.environ['CARGO_TARGET_DIR'] = CARGO_TARGET_DIR

with io.open(os.path.join(PACKAGE_PATH, VERSION)) as f:
    version = f.readline()

with io.open(README, "rt", encoding="utf8") as f:
    readme = f.read()

setup(name=PACKAGE_NAME,
      version=version,
      description='Python wrapper of the Rust Snips NLU engine',
      long_description=readme,
      author='Thibaut Lorrain, Adrien Ball',
      author_email='thibaut.lorrain@snips.ai, adrien.ball@snips.ai',
      classifiers=[
          "Programming Language :: Python :: 2",
          "Programming Language :: Python :: 2.7",
          "Programming Language :: Python :: 3",
          "Programming Language :: Python :: 3.4",
          "Programming Language :: Python :: 3.5",
          "Programming Language :: Python :: 3.6",
          "Programming Language :: Python :: 3.7",
      ],
      install_requires=[
          "future>=0.16,<0.18",
          "pathlib>=1.0,<2.0; python_version < '3.4'",
      ],
      packages=packages,
      include_package_data=True,
      rust_extensions=[RustExtension(RUST_EXTENSION_NAME, CARGO_FILE_PATH,
                                     debug="develop" in sys.argv,
                                     binding=Binding.NoBinding)],
      zip_safe=False)
