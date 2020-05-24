use std::path::{Path, PathBuf};
use std::process::Command;

/// A script used to export meshes and armatures from Blender to stdout
pub static EXPORT_BLENDER_DATA: &'static str = r#"
import bpy

bpy.context.view_layer.objects.active = None

# Get the objects at the beginning so that we don't iterate over new ones that we
# generate such as ik-to-fk converted rigs
objects = list(bpy.context.scene.objects)

for obj in objects:
    bpy.context.view_layer.objects.active = obj
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
///
/// TODO: Integration test this
pub fn export_blender_data(blender_files: &[PathBuf]) -> Result<String, anyhow::Error> {
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
        return Err(BlenderExportError::Stderr(String::from_utf8(
            output.stderr,
        )?))?;
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn open_blender_file(file: &dyn AsRef<Path>) -> String {
    format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath="{}")
"#,
        file.as_ref().to_str().unwrap()
    )
}

/// An error while exporting data from Blender
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum BlenderExportError {
    #[error("Error while exporting data from blender: {0}")]
    Stderr(String),
}
