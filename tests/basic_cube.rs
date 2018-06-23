extern crate blender_mesh_to_json;

use std::path::Path;
use std::fs::File;
use std::env::current_dir;
use std::process::Command;
use blender_mesh_to_json::parse_meshes_from_blender_stdout;

#[test]
fn parse_data () {
    let basic_cube_blend = &abs_path("tests/basic_cube.blend");
    let install_addon = &abs_path("install-addon.py");
    let run_addon = &abs_path("run-addon.py");

    // TODO: Move the CLI spawning and parsing into `lib.rs`. In our test just verify
    // the returned mesh data

    let mut blender_output =  Command::new("blender")
        .args(&["--background", basic_cube_blend])
        .args(&["--python", run_addon])
        .arg("--")
        .output()
        .expect("Failed to execute Blender process");

    let stderr = String::from_utf8(blender_output.stderr).unwrap();
    println!("{}", stderr);

    assert_eq!(stderr, "");

    let stdout = String::from_utf8(blender_output.stdout).unwrap();

    let parsed_mesh = parse_meshes_from_blender_stdout(&stdout, None).unwrap();

    println!("{}", stdout);
    println!("{:#?}", parsed_mesh);
}

fn abs_path (path: &str) -> String {
    let path = Path::new(path);
    let mut abs_path = current_dir().unwrap();
    abs_path.push(path);

    abs_path.to_str().unwrap().to_string()
}

// TODO: write_to_file.rs test where we make sure that we write to a file instead of stdout
// if `-- --mesh-filepath="" is provided

// TODO: cli.rs test that spawns a bash script that calls a python script that iterates over
// passed in mesh names and calls bpy.ops.import_export.mesh2json(). It then tee's the output
// so that readers have an example of how to combine this with other scripts

// CLI
// STDOUT=$(blender -b --python multiple-blender-files)
// JSON = $(cat STDOUT | mesh2json)
// cat STDOUT | mesh2json > some_file.json

// bpy.ops.wm.open_mainfile( filepath = "/path/to/file.blend" )
