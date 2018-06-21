//! Blender files can have meshes such as circles, cubes, cylinders, a dragon or any other
//! 3D shape.
//!
//! A mesh can be represented as a group of vertices and data about those vertices, such as their
//! normals or UV coordinates.
//!
//! Meshes can also have metadata, such as the name of it's parent armature (useful for vertex
//! skinning).
//!
//! blender-mesh-to-json seeks to be a well tested, well documented exporter for blender mesh
//! metadata.
//!
//! You can write data to stdout or to a file. At the onset it will be geared towards @chinedufn's
//! needs - but if you have needs that aren't met feel very free to open an issue.
//!
//! @see https://docs.blender.org/manual/en/dev/modeling/meshes/introduction.html - Mesh Introduction
//! @see https://github.com/chinedufn/blender-actions-to-json - Exporting blender armatures / actions

#[macro_use]
extern crate failure;

/// Something went wrong in the Blender child process that was trying to parse your mesh data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export meshes from a `.blend` file.
    #[fail(display = "There was an issue while exporting meshes: Blender stderr output: {}", _0)]
    Stderr(String),
}

/// Configuration for how to export the meshes from your `.blend` file.
#[derive(Debug)]
pub struct ExportConfig {
    /// The filepath to write the exported mesh data to.
    pub output_filepath: Option<String>
}

/// Given a buffer of standard output from Blender we parse all of the mesh JSON that was
/// written to stdout by `blender-mesh-to-json.py`.
///
/// # Examples
///
/// ```
/// let stdout = Command::new("blender")
///     .args(&["--background", "house.blend"])
///     .args(&["--python", "run-addon.py"])
///     .arg("--")
///     .output()
///     .expect("Failed to execute Blender process");
/// let stdout = String::from_utf8(blender_output.stderr).unwrap();
/// ```
pub fn parse_meshes_from_blender_stdout (stdout: &str, config: Option<&ExportConfig>) {
    // TODO: Output format {filepath: {MESH_NAME: {data}, filepath2: ...}
}
