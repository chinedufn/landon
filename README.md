blender-mesh-to-json [![npm version](https://badge.fury.io/js/blender-mesh-to-json.svg)](http://badge.fury.io/js/blender-mesh-to-json) [![Build Status](https://travis-ci.org/chinedufn/blender-mesh-to-json.svg?branch=master)](https://travis-ci.org/chinedufn/blender-mesh-to-json)
===============

> Given a Blender `.blend` file, write the joint data for all actions to a JSON file

## Initial Background / Motivation

#### Before

#### After

## To Install

```sh
cargo install  blender-mesh-to-json
# This will save the mesh2json command to your Blender User Preferences
# so that you can call it from within Blender.
mesh2json --install
```

## Usage

```
# Run via CLI
blender /path/to/my-model.blend --background --python `mesh2json` -- /path/to/output.json
```

```
# Run via bpy.ops
bpy.ops.import_export.mesh2json(filepath='/path/to/output.json')
```

The outputted file will look something like this:

```json
{
}
```

This file has ...


## CLI Usage

```sh
mesh2json --help

Usage

  $ mesh2json
    # Returns the filename of the Blender addon. Useful for running the addon via CLI
    # i.e.
    #   blender my-model.blend --python $(mesh2json) -- --filepath=/var/tmp/output-file.json

  $ mesh2json --help
    # Prints some help text on how to use this command

  $ mesh2json --install
    # Installs and enables the addon and then saves it to your Blender user preferences
    # If `blender` is not in your path do `BLENDER=/path/to/blender mesh2json --install`

Options

  -h, --help            -> Get help text about using the blender-mesh-to-json CLI

  -i, --install         -> Install the addon and save it in your Blender

```

## Note

`blender-mesh-to-json` will only export data for one mesh.

**This script currenly requires that your `bpy.context.active_object` is your mesh.**

`blender-mesh-to-json` will use the first mesh that it finds if the `bpy.context.active_object` is not a mesh,
so if your file has only one mesh you're good to go.

Otherwise you will need either need to select your desired mesh before running this script.

You can do this by either:

1. Running your own mesh selection script before this script
2. OR manually right click it while in object mode before running this script.

If you're looking to use this script as part of a fully automated pipeline, `#2` is not an option and you will need to do `#1`.

Personally, I run a single blender script that iterates over all meshes, selecting them one at a time and running `bpy.ops.import_export.mesh2json(filepath=...)`
for each mesh.

If any of this is confusing please open an issue and I'll try to give a better explanation based on your question(s)!

## To test

In order to run the tests you'll need to set an environment variable with the path to your `blender` program or
have `blender` available from your $PATH so that we can spawn a headless blender process.js.

To add Blender to your $PATH on mac you can try running `export PATH="$PATH:/Applications/blender.app/Contents/MacOS"` in your terminal.

```sh
git clone https://github.com/chinedufn/blender-mesh-to-json
cd blender-mesh-to-json
# You can exclude the Blender=... if blender is in your $PATH
BLENDER=/Applications/blender.app/Contents/MacOS/blender cargo test
```

## See Also

- [blender-iks-to-fks](https://github.com/chinedufn/blender-iks-to-fks)
- [blender-actions-to-json](https://github.com/chinedufn/blender-actions-to-json)

## License

MIT
