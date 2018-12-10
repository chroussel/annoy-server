#!/bin/bash

DIR_REGISTRY="/Users/c.roussel/.cargo/registry"

docker run \
--rm \
-it \
-v $DIR_REGISTRY:/root/.cargo/registry \
-v $(pwd):/app \
-w /app \
-t rust_build_server \
/bin/bash -c "source /opt/rh/llvm-toolset-7/enable; cargo build --release --target x86_64-unknown-linux-gnu"