use std::path::Path;
use std::fs::File;
use std::env::current_dir;
use std::process::Command;

#[test]
fn parse_data () {
    let basic_cube_blend = &abs_path("tests/basic_cube.blend");
    let install_addon = &abs_path("install-addon.py");
    let run_addon = &abs_path("run-addon.py");

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

    println!("{}", stdout);

    // TODO: Breadcrumb - spawn out blender script, look at stdout and verify that
    // our script ran
}

fn abs_path (path: &str) -> String {
    let path = Path::new(path);
    let mut abs_path = current_dir().unwrap();
    abs_path.push(path);

    abs_path.to_str().unwrap().to_string()
}

fn test_script<'a> () -> &'a str {
    r#"

    "#
}