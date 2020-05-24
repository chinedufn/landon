use crate::{export_blender_data, Subcommand};
use std::path::PathBuf;

/// Export meshes and armatures from Blender files to stdout
#[derive(Debug, StructOpt)]
#[structopt(usage = USAGE)]
pub struct ExportCmd {
    #[structopt(short = "f", long = "file")]
    file: Vec<PathBuf>,
}

impl Subcommand for ExportCmd {
    fn run(&self) -> Result<(), anyhow::Error> {
        println!("{}", export_blender_data(&self.file)?);
        Ok(())
    }
}

const USAGE: &'static str = r#"# Prints mesh and armature data to stdout. You'll typically parse that output using:
landon export -f /path/to/file1.blend -f /path/to/file2.blend
landon export -f /path/to/fil3.blend | landon parse --mode filename
landon export --help
"#;
