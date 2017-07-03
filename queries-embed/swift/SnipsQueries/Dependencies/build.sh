#!/bin/sh

 : ${PROJECT_DIR:?"${0##*/} must be invoked as part of an Xcode script phase"}

set -e

VERSION="0.20.0"
SYSTEM=$1

if [ $SYSTEM != ios ] && [ $SYSTEM != macos ]; then
    echo Given system should be ios or macos
    exit 1
fi

if [ ! -e $PROJECT_DIR/Dependencies/$SYSTEM/libsnips_queries.a ] ||
   [ ! -e $PROJECT_DIR/Dependencies/$SYSTEM/libsnips_queries.h ]; then
    echo Will download snips_queries_$SYSTEM_$VERSION
    URL=https://s3.amazonaws.com/snips/snips-queries-dev/snips-queries-$SYSTEM-$VERSION.tgz
    mkdir -p $PROJECT_DIR/Dependencies/$SYSTEM
    cd $PROJECT_DIR/Dependencies/$SYSTEM
    curl -s $URL | tar zxv
    echo Downloaded libsnips_queries $VERSION
fi
