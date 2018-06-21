use std::path::Path;
use std::fs::File;
use std::env::current_dir;

#[test]
fn parse_data () {
    let blender_mesh_exporter = Path::new("blender-mesh-to-json.py");
    let mut export_script = current_dir().unwrap();
    export_script.push(blender_mesh_exporter);

    let mut file = File::open(&export_script).unwrap();
    println!("{}", export_script.to_str().unwrap());

    println!("hi");
}