#!/bin/bash

# https://github.com/LaurentMazare/tch-rs
# https://docs.rs/crate/torch-sys/0.3.0/source/build.rs

cd "$HOME" || exit
VERSION="1.13.1"
AMD="https://download.pytorch.org/libtorch/rocm5.4.2/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Brocm5.4.2.zip"
wget -c "$AMD" -O libtorch.zip
# https://download.pytorch.org/libtorch/cu102/libtorch-cxx11-abi-shared-with-deps-${VERSION}.zip
unzip libtorch.zip

export LIBTORCH=$HOME/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
