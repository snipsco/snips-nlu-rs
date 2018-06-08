#!/bin/sh -ex

 : ${PROJECT_DIR:?"${0##*/} must be invoked as part of an Xcode script phase"}

set -e

VERSION="0.57.0"
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
    local filename=snips-nlu-${SYSTEM}.${VERSION}.tgz
    local url=https://s3.amazonaws.com/snips/snips-nlu-dev/${filename}

    rm -f ${OUT_DIR}/*
    if ! core_is_present; then
        echo "Will download '${filename}'"
        $(cd ${OUT_DIR} && curl -s ${url} | tar zxv)
    fi
}

install_local_core () {
    # TODO: Find a better way to retrieve root_dir
    local root_dir=${PROJECT_DIR}/../../../
    local target_dir=${root_dir}/target/

    rm -f ${OUT_DIR}/*
    if [ ${SYSTEM} = macos ]; then
        echo "Using macOS local build"

        cp ${target_dir}/${TARGET_BUILD_TYPE}/${LIBRARY_NAME_A} ${OUT_DIR}

    elif [ ${SYSTEM} == ios ]; then
        echo "Using iOS local build"
        local archs_array=( ${ARCHS} )

        for arch in "${archs_array[@]}"; do
            echo ${arch}
            if [ ${arch} = arm64 ]; then
                local arch=aarch64
            fi
            local library_path=${target_dir}/${arch}-apple-ios/${TARGET_BUILD_TYPE}/${LIBRARY_NAME_A}
            if [ ! -e ${library_path} ]; then
                return 1
            fi
            cp ${library_path} ${OUT_DIR}/${LIBRARY_NAME}-${arch}.a
        done

        lipo -create `find ${OUT_DIR}/${LIBRARY_NAME}-*.a` -output ${OUT_DIR}/${LIBRARY_NAME_A}

    else
        echo "${SYSTEM} isn't supported"
        return 1
    fi

    cp ${PROJECT_DIR}/../../c/${LIBRARY_NAME_H} ${OUT_DIR}
    cp ${PROJECT_DIR}/../../c/module.modulemap ${OUT_DIR}

    return 0
}

core_is_present () {
    if [ -e ${OUT_DIR}/module.modulemap ] &&
       [ -e ${OUT_DIR}/${LIBRARY_NAME_A} ] &&
       [ -e ${OUT_DIR}/${HEADER_NAME_H} ]; then
        return 0
    fi

    return 1
}

if [ "${SNIPS_USE_LOCAL}" == 1 ]; then
    install_local_core && exit 0
else
    if core_is_present; then
        exit 0
    fi

    if ! install_local_core; then
        install_remote_core && exit 0
    fi
fi
