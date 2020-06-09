use std::path::{Path, PathBuf};

/// Given some relative path such as `src/foo/bar.rs`, get the absolute path to that file
/// if it were to be relative to the workspace's root dir...
///    -> /path/to/akigi/src/foo/bar.rs`
///
/// ## Examples
///
/// ```
/// use filesystem::rel_workspace;
/// rel_workspace("crates/akigi-cli/src/lib.rs");
/// ```
pub fn rel_workspace(path: &str) -> PathBuf {
    let workspace = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let workspace = PathBuf::from(workspace).canonicalize().unwrap();

    let path = Path::new(path);

    let path = match path.strip_prefix("/") {
        Ok(path) => path,
        Err(_) => path,
    };

    workspace.join(path)
}

pub fn rel_workspace_string(rel_path: &dyn AsRef<Path>) -> String {
    rel_workspace(rel_path.as_ref().to_str().unwrap())
        .to_str()
        .unwrap()
        .to_string()
}

pub fn armature_to_json_py() -> PathBuf {
    rel_workspace("blender-armature-to-json.py")
}

pub fn bone_groups_blend() -> PathBuf {
    rel_workspace("crates/blender-export-test/src/tests/bone_groups.blend")
}

// landon repository root directory
pub fn workspace_root() -> PathBuf {
    let workspace_root = env!("CARGO_MANIFEST_DIR").to_owned() + "/../../";
    let workspace_root = PathBuf::from(workspace_root);
    workspace_root.canonicalize().unwrap()
}

pub fn blender_export_test_fixtures() -> PathBuf {
    workspace_root().join("crates/blender-export-test/src/tests")
}
