use crate::filesystem::{armature_to_json_py, bone_groups_blend, rel_workspace};
use crate::set_active_object_by_name;
use crate::tests::blender_process::BlenderRunner;
use blender_armature::parse_armatures_from_blender_stdout;
use std::process::Command;

/// Verify that we export all of the armature's bone groups
#[test]
fn exports_bone_groups() -> Result<(), anyhow::Error> {
    let mut cmd: Command = BlenderRunner {
        blender_file: bone_groups_blend(),
        cwd: rel_workspace(""),
        python_scripts: vec![
            set_active_object_by_name("BoneGroupsTest"),
            run_armature_to_json(),
        ],
    }
    .into();

    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    assert_eq!(stderr.as_str(), "");

    let parsed_armatures = parse_armatures_from_blender_stdout(&stdout);
    let parsed_armatures = blender_armature::flatten_exported_armatures(&parsed_armatures).unwrap();

    let armature = parsed_armatures.get("BoneGroupsTest").unwrap();

    let bone_groups = armature.bone_groups();

    assert_eq!(bone_groups.len(), 2);
    assert_eq!(bone_groups.get("LowerBody").unwrap(), &vec![0, 1]);
    assert_eq!(bone_groups.get("UpperBody").unwrap(), &vec![2, 3]);

    Ok(())
}

/// Install the blender-armature addon temporarily (without saving preferences) and then
/// run the armature to json script.
fn run_armature_to_json() -> String {
    format!(
        r#"
import bpy
import os
        
bpy.ops.preferences.addon_install(filepath="{}")
bpy.ops.preferences.addon_enable(module='blender-armature-to-json')

print("Running armature2json")
# Run our addon
bpy.ops.import_export.armature2json()
"#,
        armature_to_json_py().to_str().unwrap()
    )
}
