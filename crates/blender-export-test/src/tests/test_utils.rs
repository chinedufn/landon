use crate::filesystem::rel_workspace_string;
use blender_mesh::{
    flatten_exported_meshes_owned, parse_meshes_from_blender_stdout, FlattenedExportedMeshes,
};
use std::path::PathBuf;
use std::process::Command;

pub fn export_meshes_from_blender_file(blender_file: PathBuf) -> FlattenedExportedMeshes {
    let run_addon = &rel_workspace_string(&"run-addon.py");

    let blender_output = Command::new("blender")
        .arg(blender_file)
        .arg("--background")
        .args(&["--python", run_addon])
        .arg("-noaudio")
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stderr = String::from_utf8(blender_output.stderr).unwrap();
    assert_eq!(stderr, "");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    flatten_exported_meshes_owned(parse_meshes_from_blender_stdout(&stdout)).unwrap()
}
