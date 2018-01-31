from __future__ import print_function

import io
import os
import subprocess
import sys

from setuptools import setup, find_packages
from setuptools.dist import Distribution
from wheel.bdist_wheel import bdist_wheel

from rust_build import build_rust_cmdclass, RustInstallLib

# Hack to pass custom parameters because setup.py is badly documented and
# I'm fed up with this crap
script_args = [i for i in sys.argv[1:] if "build-mode" not in i]
debug = "--build-mode=release" not in sys.argv


class RustNLUDistribution(Distribution):
    def __init__(self, *attrs):
        Distribution.__init__(self, *attrs)
        self.cmdclass['install_lib'] = RustInstallLib
        self.cmdclass['bdist_wheel'] = RustBdistWheel
        self.cmdclass['build_rust'] = build_rust_cmdclass(debug=debug)

        print("Building in {} mode".format("debug" if debug else "release"))


class RustBdistWheel(bdist_wheel):
    def finalize_options(self):
        bdist_wheel.finalize_options(self)
        self.root_is_pure = False


packages = [p for p in find_packages() if
            "tests" not in p and "debug" not in p]

PACKAGE_NAME = "snips_nlu_rust"
ROOT_PATH = os.path.dirname(os.path.abspath(__file__))
PACKAGE_PATH = os.path.join(ROOT_PATH, PACKAGE_NAME)
VERSION = "__version__"

timestamp = int(subprocess.check_output(['git', 'log', '-1', '--format=%at']))

with io.open(os.path.join(PACKAGE_PATH, VERSION)) as f:
    version = f.readline().strip().replace("-SNAPSHOT", ".dev%i" % timestamp)

required = [
    "future==0.16.0"
]

setup(
    name=PACKAGE_NAME,
    version=version,
    description='Python wrapper for the Snips NLU engine',
    author='Thibaut Lorrain',
    author_email='thibaut.lorrain@snips.ai',
    install_requires=required,
    packages=packages,
    include_package_data=True,
    distclass=RustNLUDistribution,
    entry_points={
        'console_scripts': ['debug=snips_nlu_rust.debug:main_debug'],
    },
    zip_safe=False,
    script_args=script_args,
)
