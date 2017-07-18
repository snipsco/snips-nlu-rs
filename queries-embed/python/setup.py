import io
import os
import subprocess
from setuptools.dist import Distribution

from setuptools import setup, find_packages
from wheel.bdist_wheel import bdist_wheel

from rust_build import build_rust_cmdclass, RustInstallLib


class RustNLUDistribution(Distribution):
    def __init__(self, *attrs):
        Distribution.__init__(self, *attrs)
        self.cmdclass['install_lib'] = RustInstallLib
        self.cmdclass['bdist_wheel'] = RustBdistWheel
        self.cmdclass['build_rust'] = build_rust_cmdclass(debug=False)


class RustBdistWheel(bdist_wheel):
    def finalize_options(self):
        bdist_wheel.finalize_options(self)
        self.root_is_pure = False


packages = [p for p in find_packages() if "tests" not in p]

PACKAGE_NAME = "snips_nlu_rust"
ROOT_PATH = os.path.dirname(os.path.abspath(__file__))
PACKAGE_PATH = os.path.join(ROOT_PATH, PACKAGE_NAME)
VERSION = "__version__"

timestamp = int(subprocess.check_output(
    ['git', 'log', '-1', '--date=unix', '--pretty=format:%cd']))

with io.open(os.path.join(PACKAGE_PATH, VERSION)) as f:
    version = f.readline().strip().replace("-SNAPSHOT", ".dev%i" % timestamp)

required = [
    "appdirs==1.4.3",
    "packaging==16.8",
    "pyparsing==2.2.0",
    "six==1.10.0",
]

setup(
    name=PACKAGE_NAME,
    version=version,
    description='Python wrapper for the Snips NLU engine',
    author='Thibaut Lorrain',
    author_email='thibaut.lorrain@snips.ai',
    install_requires=required,
    packages=packages,
    package_data={"": [VERSION, "dylib/*", ]},
    include_package_data=True,
    distclass=RustNLUDistribution,
    zip_safe=False,
)
