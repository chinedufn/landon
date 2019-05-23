#!/bin/bash

cd $(git rev-parse --show-toplevel)

cd mesh-visualizer

rm -rf ./out

wasm-pack build --dev --no-typescript --target web --out-dir ./out

cp ./index.html ./out/
cp -r dist ./out/
