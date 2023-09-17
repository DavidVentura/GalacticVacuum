#!/bin/bash
set -xeuo pipefail
docker run --rm  -v $PWD:/src -v ~/.cargo/:/root/.cargo v-builder cargo build
#PKG_CONFIG_SYSROOT_DIR=~/aarch64-linux-musl-native/ cargo build --target=aarch64-unknown-linux-musl 
scp target/debug/vacuuminator root@192.168.2.127:
