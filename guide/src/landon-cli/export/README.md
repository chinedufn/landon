# Exporting data

The `landon export` command exports meshes and armatures from any number of Blender files.

## The export format

A Blender process writes to stdout when it is opened and after running certain commands, so in order
to export data to stdout we must find the exported data within stdout.

To make this easy, each exported JSON mesh or armature is marked with a header and a footer.

```sh
BASIC_CUBE=/tmp/basic_cube.blend

# OFFLINE_BASIC_CUBE=crates/blender-export-test/src/tests/basic_cube.blend
# cp $OFFLINE_BASIC_CUBE $BASIC_CUBE

ONLINE_BASIC_CUBE='https://github.com/chinedufn/landon/blob/master/crates/blender-export-test/src/tests/basic_cube.blend?raw=true'
curl -L $ONLINE_BASIC_CUBE > $BASIC_CUBE
```

```sh
landon export -f $BASIC_CUBE
```

The output for this command would be similar to:

<code style='display: block; white-space: pre; overflow-x: scroll;'>
Blender 2.82 (sub 7) (hash 375c7dc4caf4 built 2020-03-12 05:31:51)
Read prefs: /Users/muusername/Library/Application Support/Blender/2.82/config/userpref.blend
found bundled python: /Applications/blender.app/Contents/Resources/2.82/python
Read blend: /tmp/basic_cube.blend
START_MESH_JSON /tmp/basic_cube.blend Cube
{"attribs": {"vertices_in_each_face": [4, 4, 4, 4, 4, 4], "positions": {"indices": [0, 1, 2, 3, 4, 7, 6, 5, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 4, 0, 3, 7], "attribute": {"data": [1.0, 0.9999999403953552, -1.0, 1.0, -1.0, -1.0, -1.0000001192092896, -0.9999998211860657, -1.0, -0.9999996423721313, 1.0000003576278687, -1.0, 1.0000004768371582, 0.999999463558197, 1.0, 0.9999993443489075, -1.0000005960464478, 1.0, -1.0000003576278687, -0.9999996423721313, 1.0, -0.9999999403953552, 1.0, 1.0], "attribute_size": 3}}, "normals": {"indices": [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5], "attribute": {"data": [0.0, 0.0, -1.0, 0.0, -0.0, 1.0, 1.0, -2.8312206268310547e-07, 4.470341252726939e-08, -2.8312206268310547e-07, -1.0, -1.0430819230577981e-07, -1.0, 2.2351744632942427e-07, -1.341104365337742e-07, 2.384185791015625e-07, 1.0, 2.086162567138672e-07], "attribute_size": 3}}, "uvs": null, "bone_influences": null}, "armature_name": null, "bounding_box": {"min_corner": [-1.7881393432617188e-07, -2.980232238769531e-07, 0.0], "max_corner": [1.000000238418579, 1.000000238418579, 1.0]}, "materials": {}, "custom_properties": {}}
END_MESH_JSON /tmp/basic_cube.blend Cube
Blender quit
</code>

> NOTE: The Parse chapter will describe how to easily extract this data via the CLI

## Headers

Note the `START_MESH_JSON` and `END_MESH_JSON` lines wrapping the exported JSON.

Each data type is wrapped in its own header/footer of a similar format.

| Data type | Header              | Footer            |
| ---       | ---                 | ---               |
| Mesh      | START_MESH_JSON     | END_MESH_JSON     |
| Armature  | START_ARMATURE_JSON | END_ARMATURE_JSON |

## Multiple Files

`landon` can export from any number of Blender files by repeating the `-f` (`--file`) flag.

```sh
landon export -f some-file.blend --file another-file.blend
```

## Writing to stdout

By default the CLI writes to stdout.

```sh
landon export -f some-file.blend | grep START_MESH_JSON
```

## Writing to a file

To write to a file simply pipe the stdout to a file.

```sh
landon export -f some-file.blend > /tmp/some-file
```
