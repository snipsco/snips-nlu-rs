#!/bin/sh -e

 : ${PROJECT_DIR:?"${0##*/} must be invoked as part of an Xcode script phase"}

set -e

VERSION="0.61.2"
SYSTEM=$(echo $1 | tr '[:upper:]' '[:lower:]')
LIBRARY_NAME=libsnips_nlu_ffi
LIBRARY_NAME_A=${LIBRARY_NAME}.a
LIBRARY_NAME_H=libsnips_nlu.h
OUT_DIR=${PROJECT_DIR}/Dependencies/${SYSTEM}

if [ -z "$TARGET_BUILD_TYPE" ]; then
TARGET_BUILD_TYPE=$(echo ${CONFIGURATION} | tr '[:upper:]' '[:lower:]')
fi

if [ "${SYSTEM}" != "ios" ] && [ "${SYSTEM}" != "macos" ]; then
    echo "Given system should be 'ios' or 'macos'."
    exit 1
fi

mkdir -p ${OUT_DIR}

install_remote_core () {
    echo "Trying remote installation"

    local filename=snips-nlu-${SYSTEM}.${VERSION}.tgz
    local url=https://s3.amazonaws.com/snips/snips-nlu-dev/${filename}

    echo "Will download '${filename}' in '${OUT_DIR}'"
    if curl --output /dev/null --silent --head --fail "$url"; then
        $(cd ${OUT_DIR} && curl -s ${url} | tar zxv)
    else
        echo "Version ${VERSION} doesn't seem to have been released yet"
        echo "Could not find any file at '${url}'"
        echo "Please file issue on 'https://github.com/snipsco/snips-nlu-rs' if you believe this is an issue"
        return 1
    fi

    return 0
}

install_local_core () {
    echo "Trying local installation"

    # TODO: Find a better way to retrieve root_dir
    local root_dir=${PROJECT_DIR}/../../../
    local target_dir=${root_dir}/target/

    if [ ${SYSTEM} = macos ]; then
        echo "Using macOS local build"

        local library_path=${target_dir}/${TARGET_BUILD_TYPE}/${LIBRARY_NAME_A}
        if [ ! -e ${library_path} ]; then
            echo "Missing file '${library_path}'"
            return 1
        fi

        cp ${library_path} ${OUT_DIR}
        cp ${PROJECT_DIR}/../../c/${LIBRARY_NAME_H} ${OUT_DIR}
        cp ${PROJECT_DIR}/../../c/module.modulemap ${OUT_DIR}

    elif [ ${SYSTEM} = ios ]; then
        echo "Using iOS local build"
        local archs_array=( ${ARCHS} )

        for arch in "${archs_array[@]}"; do
            if [ ${arch} = arm64 ]; then
                local arch=aarch64
            fi
            local library_path=${target_dir}/${arch}-apple-ios/${TARGET_BUILD_TYPE}/${LIBRARY_NAME_A}
            if [ ! -e ${library_path} ]; then
                echo "Can't find library for arch ${arch}"
                echo "Missing file '${library_path}'"
                return 1
            fi
            cp ${library_path} ${OUT_DIR}/${LIBRARY_NAME}-${arch}.a
        done

        lipo -create $(find ${OUT_DIR}/${LIBRARY_NAME}-*.a) \
            -output ${OUT_DIR}/${LIBRARY_NAME_A}
        cp ${PROJECT_DIR}/../../c/${LIBRARY_NAME_H} ${OUT_DIR}
        cp ${PROJECT_DIR}/../../c/module.modulemap ${OUT_DIR}

    else
        echo "${SYSTEM} isn't supported"
        return 1
    fi

    return 0
}

core_is_present () {
    echo "Checking if core is present (and complete)"
    local files=(
        ${OUT_DIR}/module.modulemap
        ${OUT_DIR}/${LIBRARY_NAME_A}
        ${OUT_DIR}/${LIBRARY_NAME_H}
    )

    for file in "${files[@]}"; do
        if [ ! -e $file ]; then
            echo "Core isn't complete"
            echo "Missing file '$file'"
            return 1
        fi
    done

    echo "Core is present"
    return 0
}

core_is_up_to_date () {
    echo "Checking if core is up-to-date"

    local header_path=${OUT_DIR}/${LIBRARY_NAME_H}

    if [ -z $(grep "SNIPS_NLU_VERSION" $header_path) ]; then
        echo "SNIPS_NLU_VERSION not present. Skipping up-to-date check..."
        return 0
    fi

    local core_version=$(grep "SNIPS_NLU_VERSION" $header_path | cut -d'"' -f2)

    if [ "$core_version" = ${VERSION} ]; then
        echo "Core is up-to-date"
        return 0
    fi

    echo "Core isn't up-to-date"
    echo "Found version ${core_version}, expected version ${VERSION}"
    return 1
}

echo "Will check if core is present and up-to-date"
if core_is_present && core_is_up_to_date; then
    echo "Core seems present and up-to-date !"
    exit 0
fi

rm -f ${OUT_DIR}/*
if [ "${SNIPS_USE_LOCAL}" == 1 ]; then
    echo "SNIPS_USE_LOCAL=1 Will try local installation only"
    install_local_core && exit 0
elif [ "${SNIPS_USE_REMOTE}" == 1 ]; then
    echo "SNIPS_USE_REMOTE=1 Will try remote installation only"
    install_remote_core && exit 0
else
    if ! install_local_core; then
        echo "Local installation failed"
        install_remote_core && exit 0
    fi
fi
