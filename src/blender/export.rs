use std::process::Command;

// TODO: Rename all of the joints that now have duplicated names before exporting
// So that you don't end up with Bone.001 when you really expected Bone
//
// Or reallt the `iktofk` script should handle that
static EXPORT_BLENDER_DATA: &'static str = r#"
import bpy
bpy.context.scene.objects.active = None
# Get the objects at the beginning so that we don't iterate over new ones that we
# generate such as ik-to-fk converted rigs
objects = list(bpy.context.scene.objects)
for obj in objects:
    bpy.context.scene.objects.active = obj
    if obj.type == 'MESH':
      bpy.ops.import_export.mesh2json()
    if obj.type == 'ARMATURE':
      bpy.ops.rigging.iktofk()
      bpy.ops.import_export.armature2json()
"#;

/// Write the meshes and armatures from a vector of Blender filenames to stdout.
///
/// You'll typically use something like
///
/// ```ignore
///     blender_mesh::parse_meshes_from_blender_stdout
///     blender_armature::parse_meshes_from_blender_stdout
/// ```
///
/// to parse the exported data into the data structures that you need.
pub fn export_blender_data(blender_files: &Vec<String>) -> Result<String, String> {
    let mut blender_process = Command::new("blender");
    let blender_process = blender_process.arg("--background");

    for blender_file in blender_files {
        blender_process
            .arg("-noaudio")
            .args(&["--python-expr", &open_blender_file(blender_file)])
            .args(&["--python-expr", &EXPORT_BLENDER_DATA]);
    }

    let output = blender_process.output().unwrap();

    if output.stderr.len() > 0 {
        return Err(String::from_utf8(output.stderr).expect("Blender stderr"));
    }

    Ok(String::from_utf8(output.stdout).expect("Blender stdout"))
}

fn open_blender_file(file: &str) -> String {
    format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath="{}")
"#,
        file
    )
}
