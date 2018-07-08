extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;
use std::env::current_dir;
use std::path::Path;
use std::process::Command;

#[test]
fn parse_data() {
    let basic_cube_blend = &abs_path("tests/basic_cube.blend");
    let install_addon = &abs_path("install-addon.py");
    let run_addon = &abs_path("run-addon.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`? In our test just verify
    // the returned mesh data?

    let mut blender_output = Command::new("blender")
        .args(&["--background", basic_cube_blend])
        .args(&["--python", run_addon])
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stderr = String::from_utf8(blender_output.stderr).unwrap();
    assert_eq!(stderr, "");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    let parsed_meshes = parse_meshes_from_blender_stdout(&stdout).unwrap();

    let (filename, mesh) = parsed_meshes.iter().next().unwrap();

    let mesh = mesh.get("Cube").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

    assert_eq!(mesh, &expected_mesh)
}

fn expected_mesh_data() -> String {
    r#"{
            "vertex_positions": [ 1.0, 0.99999994, -1.0, 1.0, -1.0, -1.0, -1.0000001, -0.9999998, -1.0, -0.99999964, 1.0000004, -1.0, 1.0000005, 0.99999946, 1.0, 0.99999934, -1.0000006, 1.0, -1.0000004, -0.99999964, 1.0, -0.99999994, 1.0, 1.0 ],
            "vertex_position_indices": [ 0, 1, 2, 3, 4, 7, 6, 5, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 4, 0, 3, 7 ],
            "num_vertices_in_each_face": [ 4, 4, 4, 4, 4, 4 ],
            "vertex_normals": [ 0.0, 0.0, -1.0, 0.0, -0.0, 1.0, 1.0, -0.00000028312206, 0.000000044703413, -0.00000028312206, -1.0, -0.00000010430819, -1.0, 0.00000022351745, -0.00000013411044, 0.00000023841858, 1.0, 0.00000020861626 ],
            "vertex_normal_indices": [ 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5 ],
            "armature_name": null
        }
    "#.to_string()
}

fn abs_path(path: &str) -> String {
    let path = Path::new(path);
    let mut abs_path = current_dir().unwrap();
    abs_path.push(path);

    abs_path.to_str().unwrap().to_string()
}

// TODO: write_to_file.rs test where we make sure that we write to a file instead of stdout
// if `-- --mesh-filepath="" is provided

// TODO: cli.rs test that spawns a bash script that calls a python script that iterates over
// passed in mesh names and calls bpy.ops.import_export.mesh2json(). It then tee's the output
// so that readers have an example of how to combine this with other scripts

// CLI
// STDOUT=$(blender -b --python multiple-blender-files)
// JSON = $(cat STDOUT | mesh2json)
// cat STDOUT | mesh2json > some_file.json

// bpy.ops.wm.open_mainfile( filepath = "/path/to/file.blend" )
