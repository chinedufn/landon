//! Before this test / fix my code was panicking when I wrote multiple meshes to stdout.
//!
//! So this test just makes sure that we no longer panic when parsing multiple meshes.

extern crate blender_mesh;

use blender_mesh::parse_meshes_from_blender_stdout;
use std::env::current_dir;
use std::path::Path;
use std::process::Command;

#[test]
fn parse_file_with_multiple_meshes() {
    let multiple_meshes_blend = &abs_path("tests/multiple_meshes.blend");
    let run_addon = &abs_path("./run-addon.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`. In our test just verify
    // the returned mesh data

    let blender_output = Command::new("blender")
        .args(&["--background", multiple_meshes_blend])
        .args(&["--python-expr", &set_active_object_by_name("Mesh1")])
        .args(&["--python", run_addon])
        .args(&["--python-expr", &set_active_object_by_name("Mesh2")])
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();
    let _parsed_armatures = parse_meshes_from_blender_stdout(&stdout).unwrap();
}

fn abs_path(path: &str) -> String {
    let path = Path::new(path);
    let mut abs_path = current_dir().unwrap();
    abs_path.push(path);

    abs_path.to_str().unwrap().to_string()
}

fn set_active_object_by_name(name: &str) -> String {
    format!(
        r#"
import bpy
bpy.context.scene.objects.active = None
for obj in bpy.context.scene.objects:
    if obj.name == '{}':
        bpy.context.scene.objects.active = obj
"#,
        name
    )
}
