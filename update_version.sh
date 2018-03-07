#!/usr/bin/env bash
source ./.travis/common.sh

NEW_VERSION=$(head -n 1 __version__)
echo "Updating versions to version ${NEW_VERSION}"
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */Cargo.toml
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */**/build.gradle
perl -p -i -e "s/^VERSION=\".*\"\$/VERSION=\"$NEW_VERSION\"/g" */**/**/**/build.sh
perl -p -i -e "s/https:\/\/github\.com\/snipsco\/snips-nlu-rs\", tag = \".*\"/https:\/\/github\.com\/snipsco\/snips-nlu-rs\", tag = \"$NEW_VERSION\"/g" snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
echo "$NEW_VERSION" > snips-nlu-ffi/python/snips_nlu_rs/__version__
