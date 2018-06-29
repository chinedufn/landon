extern crate blender_mesh;
extern crate serde_json;

use std::fs;
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::process::Command;

// TODO: Make a directory for all of our temp build stuff (py scripts) so that we can delete it
// all easily when we're done by deleting the dir
fn main() {
    let mut blender_files = vec![];

    let tests_dir = PathBuf::from("../blender-mesh/tests");

    for entry in tests_dir.read_dir().expect("blender-mesh tests dir") {
        let blender_file = entry.unwrap().path().display().to_string();

        if blender_file.ends_with(".blend") {
            blender_files.push(blender_file)
        }
    }

    let install_addon = include_str!("../install-addon.py");
    let addon = include_str!("../blender-mesh-to-json.py");
    let temp_install_script = "/tmp/install-mesh-exporter.py";
    let temp_addon = "/tmp/blender-mesh-to-json.py";
    fs::write(temp_install_script, install_addon).unwrap();
    fs::write(temp_addon, addon).unwrap();

    let mut blender_process = Command::new("blender");
    let mut blender_process = blender_process
        .arg("--background")
        .args(&["--python", temp_install_script]);

    for blender_file in blender_files {
        println!("cargo:rerun-if-changed=../tests/{}", blender_file);

        let open_script = &open_blend_file(&blender_file);

        blender_process
            .args(&["--python-expr", open_script])
            .args(&["--python-expr", &export_all_meshes()]);
    }

    let blender_output = blender_process
        .output()
        .expect("Failed to execute Blender process");

    let blender_stdout = String::from_utf8(blender_output.stdout).unwrap();

    let meshes = blender_mesh::parse_meshes_from_blender_stdout(&blender_stdout).unwrap();

    let output_dir = "./dist";
    DirBuilder::new()
        .recursive(true)
        .create(output_dir)
        .unwrap();
    fs::remove_dir_all(output_dir).unwrap();
    DirBuilder::new()
        .recursive(true)
        .create(output_dir)
        .unwrap();

    for (_filename, meshes) in meshes.iter() {
        for (mesh_name, mesh) in meshes.iter() {
            let mesh_json = serde_json::to_string(mesh).unwrap();

            let mesh_json_filename = &format!("./dist/{}.json", mesh_name);
            fs::write(mesh_json_filename, mesh_json).unwrap();
        }
    }
}

fn open_blend_file<'a>(file: &str) -> String {
    format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath="{}")"#,
        file
    )
}

fn export_all_meshes() -> String {
    r#"
import bpy

bpy.context.scene.objects.active = None

for obj in bpy.context.scene.objects:
    if obj.type == 'MESH':
      bpy.context.scene.objects.active = obj
      bpy.ops.import_export.mesh2json()
    "#.to_string()
}
