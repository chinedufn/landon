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
    let textured_cube_blend = &abs_path("tests/textured_cube.blend");
    let _install_addon = &abs_path("install-addon.py");
    let run_addon = &abs_path("run-addon.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`? In our test just verify
    // the returned mesh data?

    let blender_output = Command::new("blender")
        .args(&["--background", textured_cube_blend])
        .args(&["--python", run_addon])
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();
    let stderr = String::from_utf8(blender_output.stderr).unwrap();

    eprintln!("STDOUT!!!\n\n = {}", stdout);
    eprintln!("STDERR!!!\n\n = {}", stderr);

    assert_eq!(stderr, "");

    let parsed_meshes = parse_meshes_from_blender_stdout(&stdout).unwrap();

    let (_filename, mesh) = parsed_meshes.iter().next().unwrap();

    let mesh = mesh.get("TexturedCube").unwrap();

    eprintln!("mesh = {:#?}", mesh);

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
            "vertex_uvs": [0.33313355, 0.66646683, 0.00019979005, 0.6664669, 0.00019976028, 0.3335331, 0.33313352, 0.33353317, 0.33313367, 0.33313346, 0.00019984959, 0.33313358, 0.00019976028, 0.00019987935, 0.33313346, 0.00019976028, 0.66646695, 0.00019976028, 0.6664669, 0.33313346, 0.33353317, 0.33313352, 0.3335333, 0.00019979005, 0.66646695, 0.3335331, 0.66646683, 0.66646683, 0.33353317, 0.6664669, 0.33353317, 0.33353314, 0.66686654, 0.33313352, 0.66686654, 0.00019976028, 0.9998003, 0.00019979996, 0.9998003, 0.33313352, 0.33313355, 0.6668665, 0.33313355, 0.99980015, 0.0001998595, 0.99980015, 0.00019976028, 0.66686654],
            "vertex_uv_indices": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
            "armature_name": null,
            "texture_name": "textured_cube-uv-layout",
            "bounding_box": {
                "lower_left_front": [-1.0000004, -1.0000006, -1.0],
                "upper_right_back": [1.0000005, 1.0000004, 1.0]
            }
        }
    "#.to_string()
}

fn abs_path(path: &str) -> String {
    let path = Path::new(path);
    let mut abs_path = current_dir().unwrap();
    abs_path.push(path);

    abs_path.to_str().unwrap().to_string()
}
