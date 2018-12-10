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

use clap::load_yaml;
use clap::App;

mod blender;
use self::blender::process_blender_subcommand;

/// Run the landon CLI
pub fn run() {
    let yaml = load_yaml!("cli.yml");

    let mut app = App::from_yaml(yaml);

    let mut help_text = vec![];
    app.write_long_help(&mut help_text).unwrap();
    let help_text: String = String::from_utf8(help_text).unwrap();

    let matches = app.get_matches();

    if let Some(matches) = matches.subcommand_matches("blender") {
        process_blender_subcommand(&matches);
    } else {
        println!("{}", help_text)
    }
}
