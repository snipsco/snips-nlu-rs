from setuptools import setup
from setuptools.dist import Distribution
import subprocess
import sys


from rust_build import build_rust_cmdclass, install_lib_including_rust

if sys.version_info[0] == 3:
    # Python 3
    from distutils.command.build_py import build_py_2to3 as _build_py
else:
    # Python 2
    from distutils.command.build_py import build_py as _build_py

class BuildRust(_build_py):
    def run(self):
        subprocess.check_call(["cargo +beta build"], cwd="..", shell=True)
        _build_py.run(self)

class BinaryDistribution(Distribution):
    """Distribution which always forces a binary package with platform name"""
    def has_ext_modules(foo):
        return True


setup(
    name='snips-queries',
    version='0.1.0-SNAPSHOT',  # TODO
    description='Snips Queries intent parser',
    author='Thibaut Lorrain',
    author_email='thivaut.lorrain@snips.ai',
    packages=["snips_queries"],
    install_requires=[],
    #package_data= {"snips_queries" : ["../../target/debug/libqueries_embed.so"]},
    cmdclass={
        #'build_py' : BuildRust
                'build_rust': build_rust_cmdclass('../Cargo.toml'),
                'install_lib': install_lib_including_rust
    },
    zip_safe=False,
    distclass=BinaryDistribution
)
