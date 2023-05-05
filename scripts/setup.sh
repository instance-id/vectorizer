#!/bin/bash

# https://github.com/LaurentMazare/tch-rs
# https://docs.rs/crate/torch-sys/0.3.0/source/build.rs

# cd "$HOME" || exit
# VERSION="1.13.1"
# AMD="https://download.pytorch.org/libtorch/rocm5.4.2/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Brocm5.4.2.zip"
# wget -c "$AMD" -O libtorch.zip
# # https://download.pytorch.org/libtorch/cu102/libtorch-cxx11-abi-shared-with-deps-${VERSION}.zip
# unzip libtorch.zip

# export LIBTORCH=$HOME/libtorch 
# export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH

mkdir -p resources || exit

pip install torch

# Download model:
#   ```sh
git lfs install
git -C resources clone https://huggingface.co/sentence-transformers/all-MiniLM-L12-v2
#   ```
# Prepare model:
#   ```sh
python ./utils/convert_model.py resources/all-MiniLM-L12-v2/pytorch_model.bin
#   ```
#
# For models missing the prefix in their saved weights (e.g. Distil-based models), the
# conversion needs to be updated to include this prefix so that the weights can be found:
#   ```sh
#   python ./utils/convert_model.py resources/path/to/pytorch_model.bin --prefix distilbert.
#   ```
#
# For models including a dense projection layer (e.g. Distil-based models), these weights
# need to be converted as well:
# ```sh
#   python ../utils/convert_model.py  resources/path/to/2_Dense/pytorch_model.bin --suffix
# ```
