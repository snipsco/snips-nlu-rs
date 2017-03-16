from setuptools import setup
from setuptools.dist import Distribution
import subprocess
import sys

from rust_build import build_rust_cmdclass, install_lib_including_rust

setup(
    name='snips-queries',
    version='0.1.0-SNAPSHOT',  # TODO
    description='Snips Queries intent parser',
    author='Thibaut Lorrain',
    author_email='thivaut.lorrain@snips.ai',
    packages=["snips_queries"],
    install_requires=[],
    cmdclass={
                'build_rust': build_rust_cmdclass(debug=False),
                'install_lib': install_lib_including_rust
    },
    zip_safe=False,
)
