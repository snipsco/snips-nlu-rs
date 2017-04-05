from setuptools import setup
import os
import re

from rust_build import build_rust_cmdclass, install_lib_including_rust

setup_py = os.path.abspath(os.path.dirname(__file__))

# Get the version of the package from __init__.py (through __version__)
# This snippet is borrowed from Lasagne (https://github.com/Lasagne/Lasagne)
try:
    with open(os.path.join(setup_py, 'snips_queries', '__init__.py'), 'r') as f:
        init_py = f.read()
    version = re.search('__version__ = "(.*)"', init_py).groups()[0]
except Exception:
    version = ''

setup(
    name='snips-queries',
    version=version,
    description='Snips Queries intent parser',
    author='Thibaut Lorrain',
    author_email='thibaut.lorrain@snips.ai',
    packages=["snips_queries"],
    install_requires=[
        "duckling==0.0.12",
    ],
    cmdclass={
        'build_rust': build_rust_cmdclass(debug=False),
        'install_lib': install_lib_including_rust
    },
    zip_safe=False
)
