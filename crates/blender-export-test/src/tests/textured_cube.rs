extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use crate::filesystem::rel_workspace_string;
use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;
use std::process::Command;

#[test]
fn parse_data() {
    let textured_cube_blend =
        &rel_workspace_string(&"crates/blender-export-test/src/tests/textured_cube.blend");
    let run_addon = &rel_workspace_string(&"run-addon.py");

    let blender_output = Command::new("blender")
        .arg(textured_cube_blend)
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

    let mesh = mesh.get("TexturedCube").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

    assert_eq!(mesh, &expected_mesh)
}

fn expected_mesh_data() -> String {
    r#"{
            "name": "TexturedCube",
            "multi_indexed_vertex_attributes": {
                "vertices_in_each_face": [ 4, 4, 4, 4, 4, 4 ],
                "positions": {
                    "indices": [ 0, 1, 2, 3, 4, 7, 6, 5, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 4, 0, 3, 7 ],
                    "attribute": {
                        "data": [ 1.0, 0.99999994, -1.0, 1.0, -1.0, -1.0, -1.0000001, -0.9999998, -1.0, -0.99999964, 1.0000004, -1.0, 1.0000005, 0.99999946, 1.0, 0.99999934, -1.0000006, 1.0, -1.0000004, -0.99999964, 1.0, -0.99999994, 1.0, 1.0 ],
                        "attribute_size": 3
                    }
                },
                "normals": {
                    "indices": [ 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5 ],
                    "attribute": {
                        "data": [ 0.0, 0.0, -1.0, 0.0, -0.0, 1.0, 1.0, -0.00000028312206, 0.000000044703413, -0.00000028312206, -1.0, -0.00000010430819, -1.0, 0.00000022351745, -0.00000013411044, 0.00000023841858, 1.0, 0.00000020861626 ],
                        "attribute_size": 3
                    }
                },
                "uvs": {
                    "indices": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
                    "attribute": {
                        "data": [0.33313355, 0.66646683, 0.00019979005, 0.6664669, 0.00019976028, 0.3335331, 0.33313352, 0.33353317, 0.33313367, 0.33313346, 0.00019984959, 0.33313358, 0.00019976028, 0.00019987935, 0.33313346, 0.00019976028, 0.66646695, 0.00019976028, 0.6664669, 0.33313346, 0.33353317, 0.33313352, 0.3335333, 0.00019979005, 0.66646695, 0.3335331, 0.66646683, 0.66646683, 0.33353317, 0.6664669, 0.33353317, 0.33353314, 0.66686654, 0.33313352, 0.66686654, 0.00019976028, 0.9998003, 0.00019979996, 0.9998003, 0.33313352, 0.33313355, 0.6668665, 0.33313355, 0.99980015, 0.0001998595, 0.99980015, 0.00019976028, 0.66686654],
                        "attribute_size": 2
                    }
                }
            },        
            "armature_name": null,
            "bounding_box": {
                "min_corner": [-1.0000004, -1.0000006, -1.0],
                "max_corner": [1.0000005, 1.0000004, 1.0]
            },
            "materials": {
                "Default": {
                    "base_color": {"ImageTexture": "textured_cube-uv-layout.png"},
                    "metallic": {"Uniform": 0.0},
                    "roughness": {"Uniform": 0.5}
                }
            }
        }
    "#.to_string()
}
