//! Before this test / fix my code was panicking when I wrote multiple meshes to stdout.
//!
//! So this test just makes sure that we no longer panic when parsing multiple meshes.

extern crate blender_mesh;

use crate::filesystem::rel_workspace_string;
use crate::set_active_object_by_name;
use blender_mesh::parse_meshes_from_blender_stdout;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

// --python-expr wasn't working in travis-ci on linux so writing the scripts to disk
// and using using --python instead
static SELECT_MESH_1_SCRIPT: &'static str = "/tmp/select-mesh-1.py";
static SELECT_MESH_2_SCRIPT: &'static str = "/tmp/select-mesh-2.py";
static SELECT_MESH_3_SCRIPT: &'static str = "/tmp/select-mesh-3.py";

#[test]
fn parse_file_with_multiple_meshes() {
    let multiple_meshes_blend =
        &rel_workspace_string(&"crates/blender-export-test/src/tests/multiple_meshes.blend");
    let run_addon = &rel_workspace_string(&"./run-addon.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`. In our test just verify
    // the returned mesh data

    let mut select_mesh1 = File::create(SELECT_MESH_1_SCRIPT).unwrap();
    select_mesh1
        .write_all(set_active_object_by_name("Mesh1").as_bytes())
        .unwrap();

    let mut select_mesh2 = File::create(SELECT_MESH_2_SCRIPT).unwrap();
    select_mesh2
        .write_all(set_active_object_by_name("Mesh2").as_bytes())
        .unwrap();

    let mut select_mesh3 = File::create(SELECT_MESH_3_SCRIPT).unwrap();
    select_mesh3
        .write_all(set_active_object_by_name("Mesh3").as_bytes())
        .unwrap();

    let blender_output = Command::new("blender")
        .arg(multiple_meshes_blend)
        .arg("--background")
        .args(&["--python", SELECT_MESH_2_SCRIPT])
        .args(&["--python", run_addon])
        .args(&["--python", SELECT_MESH_1_SCRIPT])
        .args(&["--python", run_addon])
        .args(&["--python", SELECT_MESH_3_SCRIPT])
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();
    let parsed_meshes = parse_meshes_from_blender_stdout(&stdout).unwrap();
    let parsed_meshes = blender_mesh::flatten_exported_meshes(&parsed_meshes).unwrap();
    assert_eq!(parsed_meshes.len(), 3);
}
