extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use crate::filesystem::rel_workspace_string;
use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;

use std::process::Command;

#[test]
fn parse_data() {
    let principled_material_texture_inputs_blend = &rel_workspace_string(
        &"crates/blender-export-test/src/tests/principled_material_texture_inputs.blend",
    );
    let run_addon = &rel_workspace_string(&"run-addon.py");

    let blender_output = Command::new("blender")
        .arg(principled_material_texture_inputs_blend)
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

    let mesh = mesh.get("CubeWithTextureInputs").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

    assert_eq!(mesh.materials(), expected_mesh.materials());

    assert_eq!(mesh, &expected_mesh)
}

fn expected_mesh_data() -> String {
    r#"{
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
                        "data": [0.000100158795, 0.52753544, 0.00009998002, 0.27758548, 0.25005, 0.27758533, 0.25005004, 0.5275353, 0.74995005, 0.52753514, 0.5, 0.5275352, 0.49999994, 0.27758527, 0.7499499, 0.27758518, 0.99990004, 0.5275351, 0.74995005, 0.52753514, 0.7499499, 0.27758518, 0.9998998, 0.27758515, 0.25004995, 0.52773535, 0.2500499, 0.7776853, 0.00009998002, 0.7776853, 0.00009998002, 0.52773535, 0.25005, 0.27758533, 0.49999994, 0.27758527, 0.5, 0.5275352, 0.25005004, 0.5275353, 0.5001999, 0.52773535, 0.5001999, 0.7776853, 0.25024998, 0.7776853, 0.2502499, 0.5277354],
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
                "Metal": {
                    "base_color": {"ImageTexture": "metal-material.jpg"},
                    "metallic": {"ImageTexture": ["metal-material.jpg", "G"]},
                    "roughness": {"ImageTexture": ["metal-material.jpg", "R"]}
                }
            },
            "custom_properties": {}
        }
    "#.to_string()
}
