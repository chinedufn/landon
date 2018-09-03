# Introduction

`blender-exporter` is a heavily unit and integration tested collection of tools and APIs for exporting data from Blender into Rust structs that have methods that are useful for 3D applications.

You'll typically spawn a Blender process, run a `blender-exporter` Blender python script (optionally supplemented with your own script(s)) that prints the data that you need to `stdout`
and then parse this data into one of `blender-exporter`'s types such as `BlenderMesh` or `BlenderArmature`.

```
// ...
let meshes = blender_mesh::parse_meshes_from_blender_stdout(&blender_stdout).unwrap();
let armatures = blender_armature::parse_armatures_from_blender_stdout(&blender_stdout).unwrap();
// ...
```

## Goals

- Make it as easy as possible to take something from Blender and render it in your application without compromising on flexibility.
  - We favor unopinionated APIs supplemented guides on best practices so that you have the flexibility to do what you need to do if our best practices don't fit your use case.

## Example Use Cases

Some examples of things that `blender-exporter` might help you do include:

- Export all of the meshes in a `.blend` file into a collection of `BlenderMesh`'s and call methods to get the vertex data such as positions, uvs and normals from that `BlenderMesh`.

- Export all of the armatures in a `.blend` file and call methods to get the interpolated joint data at a certain keyframe to power your skeletal animation.
