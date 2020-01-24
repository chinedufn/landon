extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use crate::filesystem::rel_workspace_string;
use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;

use std::process::Command;

#[test]
fn parse_data() {
    let principled_material_single_channel_blend = &rel_workspace_string(
        &"crates/blender-export-test/src/tests/principled_material_normal_map.blend",
    );
    let run_addon = &rel_workspace_string(&"run-addon.py");

    let blender_output = Command::new("blender")
        .arg(principled_material_single_channel_blend)
        .arg("--background")
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stderr = String::from_utf8(blender_output.stderr).unwrap();
    assert_eq!(stderr, "");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    let parsed_meshes = parse_meshes_from_blender_stdout(&stdout).unwrap();

    let (_filename, mesh) = parsed_meshes.iter().next().unwrap();

    let mesh = mesh.get("Cube").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

    assert_eq!(mesh.materials(), expected_mesh.materials());

    assert_eq!(mesh, &expected_mesh)
}

fn expected_mesh_data() -> String {
    r#"{
            "vertex_positions": [ 1.0, 0.99999994, -1.0, 1.0, -1.0, -1.0, -1.0000001, -0.9999998, -1.0, -0.99999964, 1.0000004, -1.0, 1.0000005, 0.99999946, 1.0, 0.99999934, -1.0000006, 1.0, -1.0000004, -0.99999964, 1.0, -0.99999994, 1.0, 1.0 ],
            "vertex_position_indices": [ 0, 1, 2, 3, 4, 7, 6, 5, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 4, 0, 3, 7 ],
            "num_vertices_in_each_face": [ 4, 4, 4, 4, 4, 4 ],
            "vertex_normals": [ 0.0, 0.0, -1.0, 0.0, -0.0, 1.0, 1.0, -0.00000028312206, 0.000000044703413, -0.00000028312206, -1.0, -0.00000010430819, -1.0, 0.00000022351745, -0.00000013411044, 0.00000023841858, 1.0, 0.00000020861626 ],
            "vertex_normal_indices": [ 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5 ],
            "armature_name": null,
            "bounding_box": {
                "min_corner": [-1.0000004, -1.0000006, -1.0],
                "max_corner": [1.0000005, 1.0000004, 1.0]
            },
            "materials": {
                "Material": {
                    "base_color": {
                      "Uniform": [0.800000011920929, 0.800000011920929, 0.800000011920929]
                    },
                    "metallic": {"Uniform": 0.0},
                    "roughness": {"Uniform": 0.5},
                    "normal_map": "1x1-green-pixel.png"
                }
            }
        }
    "#.to_string()
}
