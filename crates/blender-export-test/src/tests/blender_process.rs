use std::path::PathBuf;
use std::process::Command;

/// Used to spawn a blender process that runs the provided scripts in the background
#[derive(Debug)]
pub struct BlenderRunner {
    pub blender_file: PathBuf,
    pub cwd: PathBuf,
    pub python_scripts: Vec<String>,
}

impl Into<Command> for BlenderRunner {
    fn into(self) -> Command {
        let mut cmd = Command::new("blender");

        cmd.arg(self.blender_file.as_path())
            .arg("--background")
            .current_dir(self.cwd);

        for python_script in self.python_scripts.iter() {
            cmd.args(&["--python-expr", python_script]);
        }

        cmd.arg("-noaudio");

        cmd
    }
}
