#!/bin/bash
set -ev

source .travis/common.sh

if [ "${RUST_TESTS}" == "true" ]; then
    echo "Running rust tests..."
    # Uncomment when snips-nlu-resources-packed will be replaced by an other solution that take less memory
    cargo test --all || die "Rust tests failed"
    cargo run -p snips-nlu-lib \
        --example weather \
        snips-nlu-lib/examples/trained_assistant.json \
        "What will be the weather in London tomorrow at 8am?" \
        || die "Rust example failed"
fi

if [ "${PYTHON_TESTS}" == "true" ]; then
    echo "Running python tests..."
    cd snips-nlu-ffi/python
    python -m pip install tox
    tox || die "Python tests failed"
    cd -
fi

if [ "${KOTLIN_TESTS}" == "true" ]; then
    echo "Running kotlin tests..."
    cargo build -p snips-nlu-ffi
    cd snips-nlu-ffi/kotlin
    ./gradlew -Pdebug -PrustTargetPath=../../target test --info
    cd -
fi

if [ "${MACOS_SWIFT_TESTS}" == "true" ]; then
    echo "Running macOS swift tests..."
    cargo build -p snips-nlu-ffi
    cd snips-nlu-ffi/swift
    mkdir -p build/DerivedData
    set -o pipefail && xcodebuild \
        -IDECustomDerivedDataLocation=build/DerivedData \
        -workspace SnipsNlu.xcworkspace \
        -scheme SnipsNlu-macOS \
        TARGET_BUILD_TYPE=debug \
        clean \
        test \
        | xcpretty
    cd -
fi

if [ "${IOS_SWIFT_TESTS}" == "true" ]; then
    echo "Running iOS swift tests..."
    TARGET_SYSROOT=$(xcrun --sdk iphonesimulator --show-sdk-path) \
      cargo build -p snips-nlu-ffi --target x86_64-apple-ios
    cd snips-nlu-ffi/swift
    mkdir -p build/DerivedData
    set -o pipefail && xcodebuild \
        -IDECustomDerivedDataLocation=build/DerivedData \
        -workspace SnipsNlu.xcworkspace \
        -scheme SnipsNlu-iOS \
        -destination 'platform=iOS Simulator,name=iPhone 8,OS=latest' \
        TARGET_BUILD_TYPE=debug \
        clean \
        test \
        | xcpretty
    cd -
fi
