//! Before this test / fix my code was panicking when I wrote multiple meshes to stdout.
//!
//! So this test just makes sure that we no longer panic when parsing multiple meshes.

extern crate blender_armature;

use crate::filesystem::rel_workspace_string;
use crate::set_active_object_by_name;
use blender_armature::parse_armatures_from_blender_stdout;

use std::fs::File;
use std::io::Write;

use std::process::Command;

// --python-expr wasn't working in travis-ci on linux so writing the scripts to disk
// and using using --python instead
static SELECT_ARMATURE_1_SCRIPT: &'static str = "/tmp/select-armature-1.py";
static SELECT_ARMATURE_2_SCRIPT: &'static str = "/tmp/select-armature-2.py";
static SELECT_ARMATURE_3_SCRIPT: &'static str = "/tmp/select-armature-3.py";

#[test]
fn parse_file_with_multiple_armatures() {
    let multiple_armatures_blend =
        &rel_workspace_string(&"crates/blender-export-test/src/tests/multiple_armatures.blend");
    let run_addon = &rel_workspace_string(&"./blender-armature/run-armature-to-json.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`. In our test just verify
    // the returned armature data

    let mut select_armature1 = File::create(SELECT_ARMATURE_1_SCRIPT).unwrap();
    select_armature1
        .write_all(set_active_object_by_name("Armature1").as_bytes())
        .unwrap();

    let mut select_armature2 = File::create(SELECT_ARMATURE_2_SCRIPT).unwrap();
    select_armature2
        .write_all(set_active_object_by_name("Armature2").as_bytes())
        .unwrap();

    let blender_output = Command::new("blender")
        .arg(multiple_armatures_blend)
        .arg("--background")
        .args(&["--python", SELECT_ARMATURE_2_SCRIPT])
        .args(&["--python", run_addon])
        .args(&["--python", SELECT_ARMATURE_1_SCRIPT])
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    let parsed_armatures = parse_armatures_from_blender_stdout(&stdout).unwrap();
    let parsed_armatures = blender_armature::flatten_exported_armatures(&parsed_armatures).unwrap();
    assert_eq!(parsed_armatures.len(), 2);
}
