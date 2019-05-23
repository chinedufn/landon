#!/bin/bash

cd $(git rev-parse --show-toplevel)

watchexec -r \
  -w mesh-visualizer/src \
  -w mesh-visualizer/build.rs \
  -w mesh-visualizer/index.html \
    ./mesh-visualizer/build.sh
