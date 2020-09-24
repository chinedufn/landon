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
#[cfg(feature = "cli")]
extern crate structopt;

#[macro_use]
extern crate serde;

mod blender;

pub use self::blender::*;

#[cfg(feature = "cli")]
mod subcommands;

#[cfg(feature = "cli")]
pub use self::cli::*;

#[cfg(feature = "cli")]
mod cli {
    use crate::subcommands::export::ExportCmd;
    use crate::subcommands::install::InstallCmd;
    use structopt::StructOpt;

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
            };
            cmd.run()
        }
    }

    /// The Landon CLI
    /// documentation: https://chinedufn.github.io/landon/landon-cli/index.html
    #[derive(Debug, StructOpt)]
    #[structopt(name = "landon", rename_all = "kebab-case")]
    pub enum Landon {
        /// Export meshes and armatures from your Blender files to stdout as JSON
        Export(ExportCmd),
        /// Install various Blender addons
        Install(InstallCmd),
    }

    #[cfg(feature = "cli")]
    pub(crate) trait Subcommand {
        /// Run a subcommand
        fn run(&self) -> Result<(), anyhow::Error>;
    }
}

/// Attempting to run the CLI without the CLI feature.
#[cfg(not(feature = "cli"))]
pub fn run() -> ! {
    unimplemented!(
        r#"Please enable the "cli" feature.

For example:

cargo install -f landon --features cli"#
    )
}
