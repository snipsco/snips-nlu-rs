#!/bin/bash
set -ev

export PATH="$HOME/.cargo/bin:$PATH"

if [[ "${RUST_TESTS}" == "true" ]]; then
    echo "Running rust tests..."
    cargo test --all
    cargo check --benches
fi

if [[ "${PYTHON_TESTS}" == "true" ]]; then
    echo "Running python tests..."
    cd platforms/python
    python -m pip install tox
    tox
    cd -
fi

if [[ "${KOTLIN_TESTS}" == "true" ]]; then
    echo "Running kotlin tests..."
    cargo build -p snips-nlu-ffi
    cd platforms/kotlin
    ./gradlew -Pdebug -PrustTargetPath=../../target test --info
    cd -
fi

if [[ "${MACOS_SWIFT_TESTS}" == "true" ]]; then
    echo "Running macOS swift tests..."
    cargo build -p snips-nlu-ffi
    cd platforms/swift
    mkdir -p build/DerivedData
    set -o pipefail && xcodebuild \
        -IDECustomDerivedDataLocation=build/DerivedData \
        -workspace SnipsNlu.xcworkspace \
        -scheme SnipsNlu-macOS \
        TARGET_BUILD_TYPE=debug \
        SNIPS_USE_LOCAL=1 \
        clean \
        test \
        | xcpretty
    cd -
fi

if [[ "${IOS_SWIFT_TESTS}" == "true" ]]; then
    echo "Running iOS swift tests..."
    TARGET_SYSROOT=$(xcrun --sdk iphonesimulator --show-sdk-path) \
      cargo build -p snips-nlu-ffi --target x86_64-apple-ios
    cd platforms/swift
    mkdir -p build/DerivedData
    set -o pipefail && xcodebuild \
        -IDECustomDerivedDataLocation=build/DerivedData \
        -workspace SnipsNlu.xcworkspace \
        -scheme SnipsNlu-iOS \
        -destination 'platform=iOS Simulator,name=iPhone 8,OS=latest' \
        TARGET_BUILD_TYPE=debug \
        SNIPS_USE_LOCAL=1 \
        clean \
        test \
        | xcpretty
    cd -
fi
