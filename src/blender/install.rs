use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::process::Command;

/// Install the blender mesh exporter addon.
///
/// This gives you access to `bpy.ops.import_export.mesh2json()` from Blender
pub fn install_mesh_to_json() -> std::io::Result<()> {
    // Write our addon to a tmp file. Our `install_mesh_to_json_script` will look for this tmp file
    // when installing the addon.
    let addon_file_path = temp_dir().join("blender-mesh-to-json.py");
    let mesh_to_json_addon = include_str!("../../blender-mesh-to-json.py");
    let mut addon_file = File::create(&addon_file_path)?;
    addon_file.write_all(mesh_to_json_addon.as_bytes()).unwrap();

    let install_mesh_to_json_script = format!(
        r#"
# Install the addon and save the user's preferences
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = r'{}'

# Install the addon, enable it and save the user's preferences so that it
# is available whenever Blender is opened in the future
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='blender-mesh-to-json')
bpy.ops.wm.save_userpref()
    "#,
        addon_file_path.display()
    );

    // TODO: Support an environment variable to override the path to the executable
    let blender_executable = "blender";
    Command::new(blender_executable)
        .arg("--background")
        .args(&["--python-expr", &install_mesh_to_json_script])
        // https://blenderartists.org/t/cannot-run-blender-on-ubuntu-server-12-04lts/614415
        .arg("-noaudio")
        .spawn()
        .expect("blender must be in your $PATH")
        .wait()
        .unwrap();

    Ok(())
}

/// Install the blender armature exporter addon.
///
/// This gives you access to `bpy.ops.import_export.armature2json()` from Blender
pub fn install_armature_to_json() -> std::io::Result<()> {
    // Write our addon to a tmp file. Our `install_armature_to_json_script` will look for this tmp file
    // when installing the addon.
    let addon_file_path = temp_dir().join("blender-armature-to-json.py");
    let armature_to_json = include_str!("../../blender-armature-to-json.py");
    let mut addon_file = File::create(&addon_file_path)?;
    addon_file.write_all(armature_to_json.as_bytes()).unwrap();

    let install_armature_to_json_script = format!(
        r#"
import bpy

addonFilePath = r'{}'

# Install the addon, enable it and save the user's preferences so that it
# is available whenever Blender is opened in the future
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='blender-armature-to-json')
bpy.ops.wm.save_userpref()
    "#,
        addon_file_path.display()
    );

    // TODO: Support an environment variable to override the path to the executable
    let blender_executable = "blender";
    Command::new(blender_executable)
        .arg("--background")
        .args(&["--python-expr", &install_armature_to_json_script])
        // https://blenderartists.org/t/cannot-run-blender-on-ubuntu-server-12-04lts/614415
        .arg("-noaudio")
        .spawn()
        .expect("blender must be in your $PATH")
        .wait()
        .unwrap();

    Ok(())
}
