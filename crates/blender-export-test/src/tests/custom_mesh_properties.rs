// TODO: Iterate over mesh.keys, ignore _RNA_UI, write the rest to a BlenderMesh.custom_properties
// Add custom_properties to docs
// custom_mesh_properties.blend
// mesh.keys()

extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use crate::filesystem::rel_workspace_string;
use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;
use std::process::Command;

#[test]
fn exports_custom_properties() {
    let cube_with_custom_props =
        &rel_workspace_string(&"crates/blender-export-test/src/tests/custom_mesh_properties.blend");
    let run_addon = &rel_workspace_string(&"run-addon.py");

    let blender_output = Command::new("blender")
        .arg(cube_with_custom_props)
        .arg("--background")
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();
    let stderr = String::from_utf8(blender_output.stderr).unwrap();

    assert_eq!(stderr, "");

    let parsed_meshes = parse_meshes_from_blender_stdout(&stdout);

    let (_filename, mesh) = parsed_meshes.iter().next().unwrap();

    let mesh = mesh.get("CubeWithCustomProperties").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

    assert_eq!(mesh, &expected_mesh)
}

fn expected_mesh_data() -> String {
    r#"{
            "multi_indexed_vertex_attributes": {
                "vertices_in_each_face": [4, 4, 4, 4, 4, 4],
                "positions": {
                    "indices": [0, 1, 3, 2, 2, 3, 7, 6, 6, 7, 5, 4, 4, 5, 1, 0, 2, 6, 4, 0, 7, 3, 1, 5],
                    "attribute": {
                        "data": [-1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0],
                        "attribute_size": 3
                    }
                },
                "normals": {
                    "indices": [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5],
                    "attribute": {
                        "data":  [-1.0, -0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, -0.0, 1.0],
                        "attribute_size": 3
                    }
                }
            },
            "bounding_box": {
                "min_corner": [-1.0, -1.0, -1.0],
                "max_corner": [1.0, 1.0, 1.0]
            },
            "materials": {},
            "custom_properties": {
                "example_float": {
                    "Float": 20.0
                },
                "example_int": {
                    "Int": 30
                },
                "example_string": {
                    "String": "Hello"
                }
            }
        }
    "#.to_string()
}
