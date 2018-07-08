blender-exporter [![Build Status](https://travis-ci.org/chinedufn/blender-exporter.svg?branch=master)](https://travis-ci.org/chinedufn/blender-exporter) [![docs](https://docs.rs/blender-exporter/badge.svg)](https://docs.rs/blender-exporter)
===============

> Blender python scripts/addons and Rust powered tooling for exporting data such as meshes and armatures from Blender

[blender-exporter cargo documentation](https://docs.rs/blender-exporter/badge.svg)

## Initial Background / Motivation

Before this module I would export blender mesh / armature data to COLLADA using blender's collada exporter,
and then parse that COLLADA into JSON.

This worked mostly well - but here and there I'd run into a model that didn't export quite right and I'd have to dig
around to figure out why.

After a year or two of this occasionally happening.. I finally decided to invest some time in writing something myself,
knowing that I'd still run into issues here and there, but they'd be issues that I'd know how to address.

The goal of `blender-exporter` is to be a minimal suite of heavily tested, well documented tooling
for getting data out of Blender and a set of functions for pre-processing that data so that you can
make use of it in your rendering pipeline.

From the beginning `blender-exporter` will be targeted towards my needs for my game [Akigi](https://akigi.com), but please
feel very free to open issues / PRs with questions / thoughts / functionality that you think might fit into `blender-exporter`.

The goal is that getting data out of Blender and into your rendering pipeline becomes easy as pie.

## Getting Started

`blender-exporter` is in need of more documentation and hand holding around how to integrate it into your pipeline.

For example, all of the tooling uses `Rust` right now, so if you want to run any of the existing pre-processing functions such
as `triangulating` your mesh you need `Rust` installed.

So we need a binary with a CLI that you can use to interface with the API without needing Rust. As well as examples of integrating
the tooling into your non-Rust application via foreign function interface... WebAssembly... etc.

But for now.. Take a look at the [mesh-visualizer`](/mesh-visualizer) directory to see a full working example of implementing skeletal
animation with models that were exported using `blender-exporter`.

## Running the mesh visualizer locally

```
git clone https://github.com/chinedufn/blender-exporter
npm start
```

Your web browser should open up with an application that allows you to visualize all of the model's in our test suite.

![Mesh visualizer demo site](/images/mesh-visualizer-example.gif)

## Contributing

Please open issues explaining your intended use case and let's see if we should or shouldn't make `blender-exporter` support it.

Also feel free to open issues with any questions / thoughts that you have!

## Usage

// TODO ...

## CLI Usage

```sh
# TODO ...
```

## To test

```sh
cargo test --all
```

## See Also

- [blender-iks-to-fks](https://github.com/chinedufn/blender-iks-to-fks)

## License

MIT
