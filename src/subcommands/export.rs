use crate::{export_blender_data, Subcommand};
use blender_armature::{parse_armatures_from_blender_stdout, ArmaturesByFilename};
use blender_mesh::{parse_meshes_from_blender_stdout, MeshesByFilename};
use std::path::PathBuf;

/// Export meshes and armatures from Blender files to stdout as JSON
#[derive(Debug, StructOpt)]
#[structopt(usage = USAGE)]
pub struct ExportCmd {
    /// The files to export from.
    /// Can be specified multiple times such as `-f foo.blend -f bar.blend`
    #[structopt(short = "f", long = "file")]
    files: Vec<PathBuf>,
}

impl Subcommand for ExportCmd {
    fn run(&self) -> Result<(), anyhow::Error> {
        let blender_stdout = export_blender_data(&self.files)?;

        let meshes = parse_meshes_from_blender_stdout(blender_stdout.as_str());
        let armatures = parse_armatures_from_blender_stdout(blender_stdout.as_str());

        serde_json::to_writer(
            std::io::stdout(),
            &MeshesAndArmaturesByFilename { meshes, armatures },
        )?;

        Ok(())
    }
}

const USAGE: &'static str = r#"# Prints mesh and armature data to stdout as JSON.

# Export to stdout
landon export -f /path/to/file1.blend -f /path/to/file2.blend

# Export to file
landon export -f /path/to/fil3.blend > some-file.json

# Full help documentation
landon export --help
"#;

#[derive(Debug, Serialize)]
struct MeshesAndArmaturesByFilename {
    meshes: MeshesByFilename,
    armatures: ArmaturesByFilename,
}
