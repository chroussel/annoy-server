#!/bin/sh
set +ev

ANNOY_VERSION='1.14.0'

mkdir -p build
(
    cd build
    wget https://github.com/spotify/annoy/archive/v$ANNOY_VERSION.tar.gz
    tar xf v$ANNOY_VERSION.tar.gz
)

mkdir -p build/include/

cp build/annoy-$ANNOY_VERSION/src/annoylib.h build/include/.
cp build/annoy-$ANNOY_VERSION/src/kissrandom.h build/include/.