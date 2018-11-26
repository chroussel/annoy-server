mkdir -p build/

(
    cd build/
    curl -O https://capnproto.org/capnproto-c++-0.7.0.tar.gz
    tar zxf capnproto-c++-0.7.0.tar.gz
    cd capnproto-c++-0.7.0
    ./configure
    make -j6 check
    sudo make install
)

