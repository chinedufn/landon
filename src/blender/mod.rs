use clap::ArgMatches;

mod install;
pub use self::install::*;

mod export;
pub use self::export::*;
use std::path::PathBuf;

/// Process `landon blender *` subcommands
///
/// @see `landon blender --help`
pub fn process_blender_subcommand(matches: &ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("install") {
        if let Some(_matches) = matches.subcommand_matches("mesh-to-json") {
            install_mesh_to_json().unwrap();
        } else if let Some(_matches) = matches.subcommand_matches("armature-to-json") {
            install_armature_to_json().unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("export") {
        let files: Vec<PathBuf> = matches
            .values_of_lossy("file")
            .unwrap()
            .into_iter()
            .map(|f| PathBuf::from(f))
            .collect();

        println!("{}", export_blender_data(&files[..]).unwrap());
    }
}
