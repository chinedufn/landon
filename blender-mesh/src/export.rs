use crate::BlenderMesh;
use failure::Fail;
use serde_json::map::Entry;
use std::collections::HashMap;

pub type MeshesByFilename = HashMap<String, MeshesByMeshName>;
pub type MeshesByMeshName = HashMap<String, BlenderMesh>;

/// Given a buffer of standard output from Blender we parse all of the mesh JSON that was
/// written to stdout by `blender-mesh-to-json.py`.
///
/// Meshes data in stdout will look like:
///
/// START_MESH_JSON /path/to/file.blend my_mesh_name
/// {...}
/// END_MESH_JSON /path/to/file.blend my_mesh_name
///
/// @see blender-mesh-to-json.py - This is where we write to stdout
pub fn parse_meshes_from_blender_stdout(
    blender_stdout: &str,
) -> Result<MeshesByFilename, failure::Error> {
    let mut filenames_to_meshes = HashMap::new();

    let mut index = 0;

    while let Some((filename_to_mesh, next_start_index)) =
        find_first_mesh_after_index(blender_stdout, index)
    {
        filenames_to_meshes.extend(filename_to_mesh);
        index = next_start_index;
    }

    Ok(filenames_to_meshes)
}

/// Convert MesheshByFilename into a HashMap<MeshName, BlenderMesh> that flattens all of the
/// meshes across all of the files into one HashMap.
///
/// This will error if there are two meshes with the same name across two or more files.
pub fn flatten_exported_meshes(
    meshes_by_filename: MeshesByFilename,
) -> Result<HashMap<&str, &BlenderMesh>, FlattenMeshError> {
    let mut flattened_meshes = HashMap::new();

    let mut duplicate_meshes: HashMap<&str, Vec<&str>> = HashMap::new();

    for (source_filename, meshes) in meshes_by_filename.iter() {
        for (mesh_name, mesh) in meshes.iter() {
            flattened_meshes.insert(mesh_name.as_str(), mesh);

            match duplicate_meshes.entry(mesh_name.as_str()) {
                Entry::Occupied(mut duplicates) => {
                    duplicates.get_mut().push(source_filename.as_str());
                }
                Entry::Vacant(filenames) => {
                    duplicates.push(source_filename.as_str());
                }
            };
        }
    }

    let duplicate_meshes = duplicate_meshes
        .iter()
        .filter(|(mesh_name, filenames)| filenames.len() > 1)
        .collect();

    if duplicate_meshes.len() > 0 {
        return Err(FlattenMeshError::DuplicateMeshNamesAcrossFiles {
            duplicates: duplicate_meshes,
        });
    }

    Ok(flattened_meshes)
}

/// An error when trying to flatten your exported data across multiple files into one HashMap of
/// mesh name to mesh data.
#[derive(Debug, Fail)]
pub enum FlattenMeshError<'a> {
    DuplicateMeshNamesAcrossFiles {
        // HashMap<MeshName, Vec<FilesThatItAppearsIn>>
        duplicates: HashMap<&'a str, Vec<&'a str>>,
    },
}

fn find_first_mesh_after_index(
    blender_stdout: &str,
    index: usize,
) -> Option<(MeshesByFilename, usize)> {
    let blender_stdout = &blender_stdout[index as usize..];

    if let Some(mesh_start_index) = blender_stdout.find("START_MESH_JSON") {
        let mut filenames_to_meshes = HashMap::new();
        let mut mesh_name_to_data = HashMap::new();

        let mesh_end_index = blender_stdout.find("END_MESH_JSON").unwrap();

        let mesh_data = &blender_stdout[mesh_start_index..mesh_end_index];

        let mut lines = mesh_data.lines();

        let first_line = lines.next().unwrap();

        let mesh_filename: Vec<&str> = first_line.split(" ").collect();
        let mesh_filename = mesh_filename[1].to_string();

        let mesh_name = first_line.split(" ").last().unwrap().to_string();

        let mesh_data: String = lines.collect();
        let mesh_data: BlenderMesh = serde_json::from_str(&mesh_data).unwrap();

        mesh_name_to_data.insert(mesh_name, mesh_data);
        filenames_to_meshes.insert(mesh_filename, mesh_name_to_data);

        return Some((filenames_to_meshes, index + mesh_end_index + 1));
    }

    return None;
}
