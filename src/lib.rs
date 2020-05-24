//! The Landon CLI
//!
//! ```ignore
//! # To install local version
//! cargo install -f --path . # In crate root
//!
//! # To install from crates.io
//! cargo install -f landon
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate structopt;

#[macro_use]
extern crate serde;

use structopt::StructOpt;

mod blender;

pub use self::blender::*;
use crate::subcommands::export::ExportCmd;
use crate::subcommands::install::InstallCmd;
use crate::subcommands::parse::ParseCmd;

mod subcommands;

/// Run the landon CLI
pub fn run() -> Result<(), anyhow::Error> {
    let landon = Landon::from_args();
    landon.run()
}

impl Subcommand for Landon {
    fn run(&self) -> Result<(), anyhow::Error> {
        let cmd: &dyn Subcommand = match self {
            Landon::Export(cmd) => cmd,
            Landon::Install(cmd) => cmd,
            Landon::Parse(cmd) => cmd,
        };
        cmd.run()
    }
}

/// The Landon CLI
#[derive(Debug, StructOpt)]
#[structopt(name = "landon", rename_all = "kebab-case")]
pub enum Landon {
    /// Export meshes and armatures from your Blender Files
    Export(ExportCmd),
    /// Install various Blender addons
    Install(InstallCmd),
    /// Parse exported Blender data from stdin or a file.
    Parse(ParseCmd),
}

trait Subcommand {
    /// Run a subcommand
    fn run(&self) -> Result<(), anyhow::Error>;
}
