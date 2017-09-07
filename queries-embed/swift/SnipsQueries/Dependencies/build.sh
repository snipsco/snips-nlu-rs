#!/bin/sh

 : ${PROJECT_DIR:?"${0##*/} must be invoked as part of an Xcode script phase"}

set -e

VERSION="0.25.1"
SYSTEM=$1

if [ $SYSTEM != ios ] && [ $SYSTEM != macos ]; then
    echo Given system should be ios or macos
    exit 1
fi

if [ ! -e $PROJECT_DIR/Dependencies/$SYSTEM/libsnips_queries.a ] ||
   [ ! -e $PROJECT_DIR/Dependencies/$SYSTEM/libsnips_queries.h ]; then
    FILENAME=snips-queries-$SYSTEM.$VERSION.tgz
    echo Download $FILENAME
    URL=https://s3.amazonaws.com/snips/snips-queries-dev/$FILENAME
    mkdir -p $PROJECT_DIR/Dependencies/$SYSTEM
    cd $PROJECT_DIR/Dependencies/$SYSTEM
    curl -s $URL | tar zxv
fi
