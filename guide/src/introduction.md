# Introduction

`landon` is a collection of heavily unit and integration tested tooling, data structures and methods for
exporting data (such as meshes and armatures) from blender preparing it for your rendering pipeline.

A typical `landon` workflow involves running the mesh/armature data export scripts
(optionally supplemented with your own Python scripts) from Blender via the CLI or Rust API.

All export scripts write json `stdout`.

`landon` provides APIs to parse this data from stdout into Rust structs - but you can also read the JSON
output yourself if you don't use Rust.

```rust
// Parsing exported landon data via the Rust API.
// Rust is not required - you can read the exported JSON data with any programming language.
let meshes = blender_mesh::parse_meshes_from_blender_stdout(&blender_stdout);
let armatures = blender_armature::parse_armatures_from_blender_stdout(&blender_stdout);
```

## Goals

- Make it as easy as possible to take something from Blender and render it in your application without straying from the raw Blender data
  - We favor exporting the data from Blender as is and then providing APIs to transform it in the different ways that you might like.

## Example Use Cases

Some examples of things that `landon` might help you do include:

- Export all of the meshes in a `.blend` file into a collection of `BlenderMesh`'s and call methods to get the vertex data such as positions, uvs and normals from that `BlenderMesh`.

- Export all of the armatures in a `.blend` file and call methods to get the interpolated joint data at a certain keyframe to power your skeletal animation.
