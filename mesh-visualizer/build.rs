
use std::collections::HashMap;
use std::fs;
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::process::Command;

// TODO: Make a directory for all of our temp build stuff (py scripts) so that we can delete it
// all easily when we're done by deleting the dir
fn main() {
    rm_and_create_dir("./dist");

    copy_texture_to_dist();

    let mut blender_files = vec![];

    let workspace_root = format!("{}/../", env!("CARGO_MANIFEST_DIR"));
    let workspace_root = PathBuf::from(workspace_root).canonicalize().unwrap();

    let tests_dir = workspace_root.join("crates/blender-export-test/src");

    for entry in tests_dir.read_dir().expect("blender-mesh tests dir") {
        let blender_file = entry.unwrap().path().display().to_string();

        if blender_file.ends_with(".blend") {
            blender_files.push(blender_file)
        }
    }

    rm_and_create_dir("/tmp/blender-export");

    let install_mesh2json = include_str!("../install-addon.py");
    let install_mesh2json_path = "/tmp/install-mesh-exporter.py";
    fs::write(install_mesh2json_path, install_mesh2json).unwrap();

    let addon = include_str!("../blender-mesh-to-json.py");
    let temp_addon = "/tmp/blender-mesh-to-json.py";
    fs::write(temp_addon, addon).unwrap();

    let addon = include_str!("../blender-armature-to-json.py");
    let temp_addon = "/tmp/blender-export/blender-armature-to-json.py";
    fs::write(temp_addon, addon).unwrap();

    let mut blender_process = Command::new("blender");
    let blender_process = blender_process
        .arg("--background")
        .arg("-noaudio")
        .args(&["--python", install_mesh2json_path])
        // TODO: An API in our root crate for writing the script to a tmp file and giving you
        // a link to it
        .args(&[
            "--python",
            workspace_root
                .join("blender-armature/install-armature-to-json.py")
                .to_str()
                .unwrap(),
        ]);

    for blender_file in blender_files {
        println!("cargo:rerun-if-changed={}", blender_file);

        let open_script = &open_blend_file(&blender_file);

        blender_process
            .args(&["--python-expr", open_script])
            .args(&["--python-expr", &export_blender_data()]);
    }

    let blender_output = blender_process
        .output()
        .expect("Failed to execute Blender process");

    let blender_stdout = String::from_utf8(blender_output.stdout).unwrap();
    let blender_stderr = String::from_utf8(blender_output.stderr).unwrap();

    if blender_stderr.len() > 0 {
        panic!("{}", blender_stderr);
    }

    let meshes = blender_mesh::parse_meshes_from_blender_stdout(&blender_stdout).unwrap();
    let armatures = blender_armature::parse_armatures_from_blender_stdout(&blender_stdout).unwrap();

    let mut mesh_names_to_models = HashMap::new();

    // TODO: A utility method exposed by blender-mesh / armature that just does this..? and maybe
    // errors if there are duplicates..?
    for (_filename, meshes) in meshes.iter() {
        for (mesh_name, mesh) in meshes.iter() {
            mesh_names_to_models.insert(mesh_name, mesh);
        }
    }

    let meshes = bincode::serialize(&mesh_names_to_models).unwrap();
    let meshes_output = workspace_root.join("mesh-visualizer/dist/meshes.bytes");
    fs::write(meshes_output.to_str().unwrap(), meshes).unwrap();

    let mut armature_names_to_data = HashMap::new();

    // TODO: A utility method exposed by blender-mesh / armature that just does this..? and maybe
    // errors if there are duplicates..?
    for (_filename, armatures) in armatures.iter() {
        for (armature_name, armature) in armatures.iter() {
            armature_names_to_data.insert(armature_name, armature);
        }
    }

    let armatures = bincode::serialize(&armature_names_to_data).unwrap();
    let armatures_output = workspace_root.join("mesh-visualizer/dist/armatures.bytes");
    fs::write(armatures_output.to_str().unwrap(), &armatures).unwrap();
}

fn copy_texture_to_dist() {
    println!(
        "cargo:rerun-if-changed=../crates/blender-export-test/src/tests/textured_cube-uv-layout.png"
    );

    fs::copy(
        "../crates/blender-export-test/src/tests/textured_cube-uv-layout.png",
        "./dist/textured_cube-uv-layout.png",
    )
    .unwrap();
}

fn rm_and_create_dir(dirname: &str) {
    DirBuilder::new().recursive(true).create(dirname).unwrap();
    fs::remove_dir_all(dirname).unwrap();
    DirBuilder::new().recursive(true).create(dirname).unwrap();
}

fn open_blend_file<'a>(file: &str) -> String {
    format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath="{}")"#,
        file
    )
}

fn export_blender_data() -> String {
    r#"
import bpy

bpy.context.view_layer.objects.active = None

for obj in bpy.context.scene.objects:
    bpy.context.view_layer.objects.active = obj

    if obj.type == 'MESH':
      bpy.ops.import_export.mesh2json()
    if obj.type == 'ARMATURE':
      bpy.ops.import_export.armature2json()
    "#
    .to_string()
}
