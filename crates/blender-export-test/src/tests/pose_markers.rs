use crate::filesystem::rel_workspace_string;
use crate::set_active_object_by_name;
use blender_armature::parse_armatures_from_blender_stdout;
use std::fs::File;
use std::io::Write;
use std::process::Command;

static SELECT_ARMATURE_SCRIPT: &'static str = "/tmp/select-pose-marker-armature.py";

/// Verify that we export pose markers
#[test]
fn exports_pose_markers() {
    let blend_file =
        &rel_workspace_string(&"crates/blender-export-test/src/tests/pose_markers.blend");
    let run_addon = &rel_workspace_string(&"./blender-armature/run-armature-to-json.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`. In our test just verify
    // the returned mesh data

    let mut select_armature = File::create(SELECT_ARMATURE_SCRIPT).unwrap();
    select_armature
        .write_all(set_active_object_by_name("PoseMarkerTest").as_bytes())
        .unwrap();

    let blender_output = Command::new("blender")
        .arg(blend_file)
        .arg("--background")
        .args(&["--python", SELECT_ARMATURE_SCRIPT])
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stderr = String::from_utf8(blender_output.stderr).unwrap();
    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    assert_eq!(
        stderr, "",
        "\n\nBLENDER STDERR = {} \n\n, BLENDER STDOUT = {}",
        stderr, stdout
    );

    let parsed_armatures = parse_armatures_from_blender_stdout(&stdout);

    let (_filename, armature) = parsed_armatures.iter().next().unwrap();

    let armature = armature.get("PoseMarkerTest").unwrap();

    let action = &armature.actions[&"Action".to_string()];
    assert_eq!(action.pose_markers().len(), 2);

    assert_eq!(action.pose_markers()[&0], "Some Pose Marker");
    assert_eq!(action.pose_markers()[&20], "Another Pose Marker");
}
