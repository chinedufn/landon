Landon [![Build Status](https://dev.azure.com/frankienwafili/landon/_apis/build/status/chinedufn.landon?branchName=master)](https://dev.azure.com/frankienwafili/landon/_build/latest?definitionId=3&branchName=master) [![Build Status](https://travis-ci.org/chinedufn/landon.svg?branch=master)](https://travis-ci.org/chinedufn/landon) [![docs](https://docs.rs/landon/badge.svg)](https://docs.rs/landon)
===============

> A collection of tools, data structures and methods for exporting Blender data (such as meshes and armatures) and preparing it for your rendering pipeline.

[The Landon Book](https://chinedufn.github.io/landon/)

- [blender-mesh API docs](https://chinedufn.github.io/landon/api/blender_mesh)

- [blender-armature API docs](https://chinedufn.github.io/landon/api/blender_armature)

- [landon API / CLI docs](https://docs.rs/landon/badge.svg)

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

`landon` is in need of more documentation and hand holding around how to integrate it into your pipeline.

For example, all of the tooling uses `Rust` right now, so if you want to run any of the existing pre-processing functions such
as `triangulating` your mesh you need `Rust` installed.

So we need a binary with a CLI that you can use to interface with the API without needing Rust. As well as examples of integrating
the tooling into your non-Rust application via foreign function interface... WebAssembly... etc.

But for now.. Take a look at the [mesh-visualizer](/mesh-visualizer) directory to see a full working example of implementing skeletal
animation with models that were exported using `landon`.

## Running the mesh visualizer locally

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

## To Install

We currently support `Blender 2.80`

```
cargo install -f landon

# Install blender mesh json exporter
landon blender install mesh-to-json

# Install blender armature json addon
landon blender install armature-to-json
```

## API

[landon](https://docs.rs/landon)

## CLI Usage

```sh
# Help on all of the subcommands
landon -h
```

```sh
# Exporting data
landon blender export -h
```

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
