#!/usr/bin/env bash

NEW_VERSION=${1?"usage $0 <new version>"}

echo "Updating versions to version ${NEW_VERSION}"
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */Cargo.toml
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */**/**/Cargo.toml
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */**/build.gradle
perl -p -i -e "s/^VERSION=\".*\"\$/VERSION=\"$NEW_VERSION\"/g" */**/**/**/build.sh
echo "$NEW_VERSION" > snips-nlu-ffi/python/snips_nlu_rust/__version__

if [[ "${NEW_VERSION}" == "${NEW_VERSION/-SNAPSHOT/}" ]]
then
    perl -p -i -e "s/snips-nlu-rs\", tag = \".*\"/snips-nlu-rs\", tag = \"$NEW_VERSION\"/g" \
        snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
    perl -p -i -e "s/snips-nlu-rs\", branch = \".*\"/snips-nlu-rs\", tag = \"$NEW_VERSION\"/g" \
        snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
else
    perl -p -i -e "s/snips-nlu-rs\", branch = \".*\"/snips-nlu-rs\", branch = \"develop\"/g" \
        snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
    perl -p -i -e "s/snips-nlu-rs\", tag = \".*\"/snips-nlu-rs\", branch = \"develop\"/g" \
        snips-nlu-ffi/python/snips-nlu-python-ffi/Cargo.toml
fi
