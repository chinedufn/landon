use crate::Subcommand;
use blender_armature::{parse_armatures_from_blender_stdout, ArmaturesByFilename};
use blender_mesh::{parse_meshes_from_blender_stdout, MeshesByFilename};
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct ParseCmd {
    /// The file that contains the exported Blender data.
    /// If no file is provided the data will be read from stdin
    file: Option<PathBuf>,
}

impl Subcommand for ParseCmd {
    fn run(&self) -> Result<(), anyhow::Error> {
        let blender_stdout = match self.file.as_ref() {
            Some(file) => std::fs::read_to_string(file)?,
            None => {
                let mut stdin = String::new();
                std::io::stdin().read_to_string(&mut stdin)?;

                stdin
            }
        };

        let meshes = parse_meshes_from_blender_stdout(blender_stdout.as_str());
        let armatures = parse_armatures_from_blender_stdout(blender_stdout.as_str());

        serde_json::to_writer(
            std::io::stdout(),
            &MeshesAndArmaturesByFilename { meshes, armatures },
        )?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct MeshesAndArmaturesByFilename {
    meshes: MeshesByFilename,
    armatures: ArmaturesByFilename,
}
