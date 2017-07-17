import io
import os
import subprocess

from setuptools import setup, find_packages

from rust_build import build_rust_cmdclass, RustInstallLib

packages = [p for p in find_packages() if "tests" not in p]

PACKAGE_NAME = "snips_queries"
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
    description='Snips Queries intent parser',
    author='Thibaut Lorrain',
    author_email='thibaut.lorrain@snips.ai',
    install_requires=required,
    packages=packages,
    package_data={
        "": [
            VERSION,
            "dylib/*",
        ]},
    include_package_data=True,
    cmdclass={
        'build_rust': build_rust_cmdclass(debug=True),
        'install_lib': RustInstallLib
    },
    zip_safe=False
)
