#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo +nightly build -p mesh-visualizer --target wasm32-unknown-unknown && \
  wasm-bindgen --no-typescript --no-modules target/wasm32-unknown-unknown/debug/mesh_visualizer.wasm  --out-dir ./mesh-visualizer
