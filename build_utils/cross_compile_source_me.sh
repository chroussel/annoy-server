# Linker for the target platform
# (cc can also be updated using .cargo/config)
export TARGET_CC="x86_64-unknown-linux-gnu-gcc"

# Library headers to link against
export TARGET_CFLAGS="-I $(pwd)/build/cc/usr/include/x86_64-linux-gnu -isystem $(pwd)/build/cc/usr/include"
# Libraries (shared objects) to link against
export LD_LIBRARY_PATH="$(pwd)/build/cc/usr/lib/x86_64-linux-gnu;$(pwd)/build/cc/lib/x86_64-linux-gnu"

# openssl-sys specific build flags
export OPENSSL_DIR="$(pwd)/build/cc/usr/"
export OPENSSL_LIB_DIR="$(pwd)/build/cc/usr/lib/x86_64-linux-gnu/"