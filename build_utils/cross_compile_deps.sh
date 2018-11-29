#!/bin/sh

mkdir -p build/cc

(
    cd build/cc
    export URL=http://security-cdn.debian.org/debian-security/pool/updates/main/o/openssl/libssl-dev_1.1.0f-3+deb9u2_amd64.deb
    curl -O $URL
    ar p $(basename $URL) data.tar.xz | tar xvf -

    export URL=http://security-cdn.debian.org/debian-security/pool/updates/main/o/openssl/libssl1.1_1.1.0f-3+deb9u2_amd64.deb
    curl -O $URL
    ar p $(basename $URL) data.tar.xz | tar xvf -

    export URL=http://ftp.us.debian.org/debian/pool/main/g/glibc/libc6_2.24-11+deb9u3_amd64.deb
    curl -O $URL
    ar p $(basename $URL) data.tar.xz | tar xvf -
)