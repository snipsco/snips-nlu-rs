#!/bin/bash
set -ev

source .travis/common.sh

echo "Replacing snips-nlu-ffi url for local version"
perl -p -i -e "s/^snips-nlu-ffi = .*\$/snips-nlu-ffi = { path = \"..\/..\" \}/g" snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
