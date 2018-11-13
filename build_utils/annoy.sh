#!/bin/sh
set +ev

ANNOY_VERSION='1.14.0'

mkdir -p build
(
    cd build
    wget https://github.com/spotify/annoy/archive/v$ANNOY_VERSION.tar.gz
    tar xf v$ANNOY_VERSION.tar.gz
    cd annoy-$ANNOY_VERSION
    g++ -undefined -bundle -archx86_64 -o build/lib/annoylib.o src/annoylib.h
)

mkdir -p annoy/include/
mkdir -p annoy/lib

cp build/annoy-$ANNOY_VERSION/src/annoylib.h annoy/include/.
cp build/annoy-$ANNOY_VERSION/build/lib*/annoy/