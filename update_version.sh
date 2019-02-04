#!/usr/bin/env bash

NEW_VERSION=${1?"usage $0 <new version>"}

echo "Updating versions to version ${NEW_VERSION}"
find . -name "Cargo.toml" -exec perl -p -i -e "s/^version = \".*\"$/version = \"$NEW_VERSION\"/g" {} \;
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" */**/build.gradle
perl -p -i -e "s/^VERSION=\".*\"\$/VERSION=\"$NEW_VERSION\"/g" */**/**/**/build.sh
perl -p -i -e "s/SNIPS_NLU_VERSION \".*\"/SNIPS_NLU_VERSION \"$NEW_VERSION\"/g" platforms/c/libsnips_nlu.h

echo "$NEW_VERSION" > platforms/python/snips_nlu_rust/__version__

if [[ "${NEW_VERSION}" == "${NEW_VERSION/-SNAPSHOT/}" ]]
then
    perl -p -i -e \
        "s/^snips-nlu-ffi = \{.*\}$/snips-nlu-ffi = { git = \"https:\/\/github.com\/snipsco\/snips-nlu-rs\", tag = \"$NEW_VERSION\" }/g" \
        platforms/python/ffi/Cargo.toml
else
    perl -p -i -e \
        "s/^snips-nlu-ffi = \{.*\}$/snips-nlu-ffi = { path = \"..\/..\/..\/ffi\" }/g" \
        platforms/python/ffi/Cargo.toml

fi
