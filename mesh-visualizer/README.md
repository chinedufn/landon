# Mesh Visualizer

While building an exporter it's important to be able to take a look at the final results
and make sure that they match what you'd expect.

This crate has a build script that runs through all of the `.blend` files in our [tests](../tests) directory and

1. Opens the `.blend` file
2. Deserializes all of the meshes and armatures in the `.blend` file into `BlenderMesh` and `BlenderArmature` structs
3. Serializes all of this data to disk via `serde` and `bincode`

We then download this data in a WebGL client, deserialize it and render it in the browser using WebGL + WebAssembly.

Using this client we're able to take a look at all of our models and verify that they render and animate as expected.

Note that this is not a substitute for good unit and integration testing, but rather a final check to be certain
that our exporter is working properly.

```sh
# To run the mesh visualizer locally
git clone https://github.com/chinedufn/landon
npm install
npm start
```

TODO: Host via `now.sh`

TODO: Screenshot of the app
