use crate::{install_armature_to_json, install_mesh_to_json, Subcommand};

#[derive(Debug, StructOpt)]
pub struct InstallCmd {
    #[structopt(short = "m", long = "mesh-to-json")]
    mesh_to_json: bool,
    #[structopt(short = "a", long = "armature-to-json")]
    armature_to_json: bool,
}

impl Subcommand for InstallCmd {
    fn run(&self) -> Result<(), anyhow::Error> {
        if self.mesh_to_json {
            install_mesh_to_json()?;
        }

        if self.armature_to_json {
            install_armature_to_json()?;
        }

        Ok(())
    }
}
