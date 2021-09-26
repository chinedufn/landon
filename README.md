Landon [![Actions Status](https://github.com/chinedufn/landon/workflows/ci/badge.svg)](https://github.com/chinedufn/landon/actions)
===============

> A collection of tools, data structures and methods for exporting Blender data (such as meshes and armatures) and preparing it for your rendering pipeline.

[The Landon Book](https://chinedufn.github.io/landon/)

- [blender-mesh API docs](https://chinedufn.github.io/landon/api/blender_mesh)

- [blender-armature API docs](https://chinedufn.github.io/landon/api/blender_armature)

- [landon API / CLI docs](https://docs.rs/landon/badge.svg)

- [Convert Ik to FK](/crates/iks-to-fks)

## Initial Background / Motivation

Before this module I would export blender mesh / armature data to COLLADA using blender's collada exporter,
and then parse that COLLADA into JSON.

This worked mostly well - but here and there I'd run into a model that didn't export quite right and I'd have to dig
around to figure out why.

After a year or two of this occasionally happening.. I finally decided to invest some time in writing something myself,
knowing that I'd still run into issues here and there, but they'd be issues that I'd know how to address.

The goal of `landon` is to be a minimal suite of heavily tested, well documented tooling
for getting data out of Blender and a set of functions for pre-processing that data so that you can
make use of it in your rendering pipeline.

From the beginning `landon` will be targeted towards my needs for my game [Akigi](https://akigi.com), but please
feel very free to open issues / PRs with questions / thoughts / functionality that you think might fit into `landon`.

The goal is that getting data out of Blender and into your rendering pipeline becomes easy as pie.

## Getting Started

[The Landon Book](https://chinedufn.github.io/landon/) is a work in progress guide with examples on how to use the libraries
in `landon`.

Take a look at the [mesh-visualizer](/mesh-visualizer) directory to see a full working example of implementing skeletal
animation with models that were exported using `landon`.

## Quick Start

Here's an example where we'll download a Blender file, export the meshes to JSON and then extract each bounding box from
the JSON using the [jq](https://stedolan.github.io/jq/) CLI.

```
# Install landon
cargo install -f landon
landon install --mesh-to-json --armature-to-json

# Download a Blender file to try landon with
BLEND_FILE='https://github.com/chinedufn/landon/blob/master/crates/blender-export-test/src/tests/multiple_meshes.blend?raw=true'
curl -L $BLEND_FILE > /tmp/multiple-meshes.blend

# Export meshes and armatures to stdout and redirect stdout to a file
landon export -f /tmp/multiple-meshes.blend > exported.json

# List all of the mesh names and bounding boxes
cat exported.json | jq -r '.meshes | to_entries[] | .value | to_entries[] | "\(.key), \(.value | .bounding_box)"'

# Second_Mesh, {"min_corner":[-1.3121787,0.44901967,0.67399526],"max_corner":[0.7619256,2.523124,2.7480996]}
# AMesh, {"min_corner":[-3.2487504,-3.3098261,1.2566323],"max_corner":[-1.2487504,-1.3098261,3.2566323]}
# Mesh3, {"min_corner":[-1.2058887,-2.4149196,-1.8447866],"max_corner":[0.86821556,-0.3408153,0.22931767]}
```

## To Install

### Install Blender
We currently support `Blender 2.80` The Blender command line executable should be installed and availble in your shell path. On MacOs you can install through homebrew to acheive this.

```
$ blender --version
Blender 2.82 (sub 7)
	build date: 2020-03-12
	build time: 05:31:51
	build commit date: 2020-03-12
	build commit time: 05:06
	build hash: 375c7dc4caf4
	build platform: Darwin
	build type: Release
	build c flags:  -Wall -Werror=implicit-function-declaration -Werror=return-type -Wno-tautological-compare -Wno-unknown-pragmas -Wno-char-subscripts -Wstrict-prototypes -Wmissing-prototypes -Wunused-parameter  -mmacosx-version-min=10.11 -Xclang -fopenmp -I'/Users/gast/blender-buildbot/mac_x86_64_10_9_cmake/blender.git/../lib/darwin/openmp/include' -std=gnu11   -msse -pipe -funsigned-char -msse2
	build c++ flags:  -Wall -Wno-tautological-compare -Wno-unknown-pragmas -Wno-char-subscripts -Wno-overloaded-virtual -Wno-sign-compare -Wno-invalid-offsetof  -mmacosx-version-min=10.11 -ftemplate-depth=1024 -stdlib=libc++ -Xclang -fopenmp -I'/Users/gast/blender-buildbot/mac_x86_64_10_9_cmake/blender.git/../lib/darwin/openmp/include' -std=c++11   -msse -pipe -funsigned-char -msse2
	build link flags: -fexceptions -framework CoreServices -framework Foundation -framework IOKit -framework AppKit -framework Cocoa -framework Carbon -framework AudioUnit -framework AudioToolbox -framework CoreAudio -framework Metal -framework QuartzCore -framework ForceFeedback -liconv -Xlinker -unexported_symbols_list -Xlinker '/Users/gast/blender-buildbot/mac_x86_64_10_9_cmake/blender.git/source/creator/osx_locals.map' -stdlib=libc++
	build system: CMake
```

### Install Landon

```
cargo install -f landon

landon install --mesh-to-json --armature-to-json
# FIXME: landon install --ik-to-fk
npm install -g ik2fk && ik2fk --install

# More info
landon install --help
```

## API

Landon provides a Rust API for exporting data programatically. 

[landon](https://docs.rs/landon)

In the future we will also provide C and Wasm APIs as light wrappers around the Rust API in order
to enable interop with other languages.


## CLI Usage

```sh
# Help on all of the subcommands
landon --help
```

## Running the mesh visualizer locally

TODO: Rewrite this example and remove watchexec as a dependency.

```
# Install a static server that sets the application/wasm mime type
npm install -g http-server
# Watcher
cargo install watchexec

git clone https://github.com/chinedufn/landon

watchexec -r -w mesh-visualizer --ignore mesh-visualizer/out ./mesh-visualizer/build.sh

http-server ./mesh-visualizer/out --open
```

Your web browser should open up with an application that allows you to visualize all of the model's in our test suite.

![Mesh visualizer demo site](/images/mesh-visualizer-example.gif)

## Contributing

Please open issues explaining your intended use case and let's see if we should or shouldn't make `landon` support it.

Also feel free to open issues with any questions / thoughts that you have!

## To test

```sh
cargo test --all
```

## TODO

- [ ] BlenderMesh's triangulate function can deal with ngons. Right now only handles 3 or 4 faces

## See Also

- [blender-iks-to-fks](https://github.com/chinedufn/blender-iks-to-fks)

## License

MIT
