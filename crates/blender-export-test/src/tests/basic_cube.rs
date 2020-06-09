extern crate blender_mesh;
extern crate serde;
extern crate serde_json;

use crate::filesystem::{rel_workspace, rel_workspace_string, workspace_root};
use blender_mesh::parse_meshes_from_blender_stdout;
use blender_mesh::BlenderMesh;

use crate::tests::test_utils::export_meshes_from_blender_file;

#[test]
fn parse_data() {
    let basic_cube_blend =
        workspace_root().join("crates/blender-export-test/src/tests/basic_cube.blend");

    let parsed_meshes = export_meshes_from_blender_file(basic_cube_blend);
    let mesh = parsed_meshes.get("Cube").unwrap();

    let expected_mesh = &expected_mesh_data();
    let expected_mesh: BlenderMesh = serde_json::from_str(expected_mesh).unwrap();

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
                }
            },
            "armature_name": null,
            "bounding_box": {
                "min_corner": [-1.7881393432617188e-07, -2.980232238769531e-07, 0.0],
                "max_corner": [1.000000238418579, 1.000000238418579, 1.0]
            },
            "materials": {},
            "custom_properties": {}
        }
    "#.to_string()
}
