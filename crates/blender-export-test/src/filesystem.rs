use std::path::{Path, PathBuf};

pub fn rel_workspace(rel_path: &dyn AsRef<Path>) -> PathBuf {
    let rel_path = rel_path.as_ref();
    let rel_path = match rel_path.strip_prefix("/") {
        Ok(rel_path) => rel_path,
        Err(_) => rel_path,
    };

    workspace_root().join(rel_path)
}

pub fn rel_workspace_string(rel_path: &dyn AsRef<Path>) -> String {
    rel_workspace(rel_path).to_str().unwrap().to_string()
}

// landon repository root directory
fn workspace_root() -> PathBuf {
    let workspace_root = env!("CARGO_MANIFEST_DIR").to_owned() + "/../../";
    let workspace_root = PathBuf::from(workspace_root);
    eprintln!("workspace_root = {:#?}", workspace_root);
    workspace_root.canonicalize().unwrap()
}
