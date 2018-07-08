#!/bin/bash

cd $(git rev-parse --show-toplevel)

cd ./mesh-visualizer

rm -rf ./site-dist
mkdir site-dist

cp ./index.html ./site-dist/
cp ./index.js ./site-dist/
cp ./bootstrap.js ./site-dist/

cargo +nightly build -p mesh-visualizer --target wasm32-unknown-unknown --release && \
  wasm-gc ../target/wasm32-unknown-unknown/release/mesh_visualizer.wasm ./site-dist/mesh_visualizer.wasm && \
  wasm-bindgen --no-typescript ./site-dist/mesh_visualizer.wasm  --out-dir ./site-dist

wasm-opt -Oz -o ./site-dist/mesh_visualizer_bg.optimized.wasm ./site-dist/mesh_visualizer_bg.wasm

wasm-gc ./site-dist/mesh_visualizer_bg.optimized.wasm -o ./site-dist/mesh_visualizer_bg.wasm

rm ./site-dist/mesh_visualizer_bg.optimized.wasm
rm ./site-dist/mesh_visualizer.wasm
