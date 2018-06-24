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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_json::Error;
use std::collections::HashMap;
use std::collections::HashSet;

/// Something went wrong in the Blender child process that was trying to parse your mesh data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export meshes from a `.blend` file.
    #[fail(display = "There was an issue while exporting meshes: Blender stderr output: {}", _0)]
    Stderr(String),
}

/// All of the data about a Blender mesh
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlenderMesh {
    /// [v1x, v1y, v1z, v2x, v2y, v2z, ...]
    pub vertex_positions: Vec<f32>,
    /// The indices within vertex positions that make up each triangle in our mesh.
    /// Three vertex position indices correspond to one triangle
    /// [0, 1, 2, 0, 2, 3, ...]
    pub vertex_position_indices: Vec<u32>,
    pub num_vertices_in_each_face: Vec<u8>,
    pub vertex_normals: Vec<f32>,
    pub vertex_normal_indices: Option<Vec<u32>>,
    pub armature_name: Option<String>,
    // TODO: textures: HashMap<TextureNameString, {uvs, uv_indices}>
}

impl BlenderMesh {
    pub fn from_json(json_str: &str) -> Result<BlenderMesh, Error> {
        serde_json::from_str(json_str)
    }
}

impl BlenderMesh {
    /// We export our models with indices for positions, normals and uvs because
    ///
    ///  1) Easier because ... todo ...
    ///  2) Reduces amount of data required to represent the model on disk
    ///
    /// OpenGL only supports one index buffer, we convert our vertex data
    /// from having three indices to having one. This usually requires some duplication of
    /// data. We duplicate the minimum amount of vertex data necessary.
    pub fn combine_vertex_indices(&mut self) {
        type PosIndex = u32;
        type NormalIndex = u32;
        type UvIndex = Option<u32>;
        type EncounteredIndices = HashSet<(PosIndex, NormalIndex, UvIndex)>;

        let mut largest_pos_index = *self.vertex_position_indices.iter().max().unwrap() as usize;

        let mut encountered_indices: EncounteredIndices = HashSet::new();
        let mut encountered_pos_indices = HashSet::new();

        let mut single_index_normals = vec![];
        let mut single_index_pos_indices = vec![];
        let mut single_index_positions = vec![];

        single_index_pos_indices.resize(self.vertex_position_indices.len(), 0);

        for (vert_num, pos_index) in self.vertex_position_indices.iter().enumerate() {
            let pos_index = *pos_index;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[vert_num];

            // FIXME: Don't reallocate an vector every iteration... Only reallocate when necessary.
            // Also trim the vector when we're done
            single_index_positions.resize(largest_pos_index * 3 + 7, 0.0);
            single_index_normals.resize(largest_pos_index * 3 + 7, 0.0);

            if encountered_indices.contains(&(pos_index, normal_index, None))
                || !encountered_pos_indices.contains(&pos_index)
            {
                single_index_pos_indices[vert_num] = pos_index;

                single_index_positions[pos_index as usize * 3] =
                    self.vertex_positions[pos_index as usize * 3];
                single_index_positions[pos_index as usize * 3 + 1] =
                    self.vertex_positions[pos_index as usize * 3 + 1];
                single_index_positions[pos_index as usize * 3 + 2] =
                    self.vertex_positions[pos_index as usize * 3 + 2];

                single_index_normals[normal_index as usize * 3] =
                    self.vertex_normals[normal_index as usize];
                single_index_normals[normal_index as usize * 3 + 1] =
                    self.vertex_normals[normal_index as usize + 1];
                single_index_normals[normal_index as usize * 3 + 2] =
                    self.vertex_normals[normal_index as usize + 2];

                encountered_pos_indices.insert(pos_index);
                encountered_indices.insert((pos_index, normal_index, None));
            } else {
                largest_pos_index += 1;

                single_index_pos_indices[vert_num] = largest_pos_index as u32;

                single_index_positions[largest_pos_index * 3] = self.vertex_positions[pos_index as usize * 3];
                single_index_positions[largest_pos_index * 3 + 1] = self.vertex_positions[pos_index as usize * 3 + 1];
                single_index_positions[largest_pos_index * 3 + 2] = self.vertex_positions[pos_index as usize * 3 + 2];

                single_index_normals[largest_pos_index as usize * 3] =
                    self.vertex_normals[normal_index as usize];
                single_index_normals[largest_pos_index as usize * 3 + 1] =
                    self.vertex_normals[normal_index as usize + 1];
                single_index_normals[largest_pos_index as usize * 3 + 2] =
                    self.vertex_normals[normal_index as usize + 2];

                encountered_indices.insert((largest_pos_index as u32, normal_index, None));
                encountered_pos_indices.insert(largest_pos_index as u32);
            }
        }

        self.vertex_position_indices = single_index_pos_indices;
        self.vertex_normals = single_index_normals;
        self.vertex_positions = single_index_positions;

        self.vertex_positions.resize(largest_pos_index * 3 + 3, 0.0);
        self.vertex_normals.resize(largest_pos_index * 3 + 3, 0.0);

        self.vertex_normal_indices = None;
    }
}

pub type MeshNamesToData = HashMap<String, BlenderMesh>;
pub type FilenamesToMeshes = HashMap<String, MeshNamesToData>;

/// Given a buffer of standard output from Blender we parse all of the mesh JSON that was
/// written to stdout by `blender-mesh-to-json.py`.
///
/// Meshes data in stdout will look like:
///
/// ```
/// START_MESH_JSON /path/to/file.blend my_mesh_name
/// END_MESH_JSON /path/to/file.blend my_mesh_name
/// ```
///
/// @see blender-mesh-to-json.py - This is where we write to stdout
pub fn parse_meshes_from_blender_stdout(
    blender_stdout: &str,
) -> Result<FilenamesToMeshes, failure::Error> {
    let mut filenames_to_meshes = HashMap::new();

    let mut index = 0;

    while let Some((filename_to_mesh, next_start_index)) =
        find_first_mesh_after_index(blender_stdout, index)
    {
        filenames_to_meshes.extend(filename_to_mesh);
        index = next_start_index;
    }

    // TODO: Breadcrumb - Plan mesh visualizer to visualizer our basic_cube.rs on paper.
    // Step 1 is adding a function to our main crate that expands our 3 vertex indices into just one.
    // Unit test it

    Ok(filenames_to_meshes)
}

fn find_first_mesh_after_index(
    blender_stdout: &str,
    index: usize,
) -> Option<(FilenamesToMeshes, usize)> {
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

        return Some((filenames_to_meshes, mesh_end_index + 1));
    }

    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_pos_norm_uv_indices() {
        let mut start_v0 = vec![0.0, 0.0, 0.0];
        let mut start_v1 = vec![1.0, 1.0, 1.0];
        let mut start_v2 = vec![2.0, 2.0, 2.0];
        let mut start_v3 = vec![3.0, 3.0, 3.0];

        let mut start_n0 = vec![4.0, 4.0, 4.0];
        let mut start_n1 = vec![5.0, 5.0, 5.0];
        let mut start_n2 = vec![6.0, 6.0, 6.0];

        let mut start_positions = vec![];
        start_positions.append(&mut start_v0);
        start_positions.append(&mut start_v1);
        start_positions.append(&mut start_v2);
        start_positions.append(&mut start_v3);

        let mut start_normals = vec![];
        start_normals.append(&mut start_n0);
        start_normals.append(&mut start_n1);
        start_normals.append(&mut start_n2);

        let mut mesh_to_combine = BlenderMesh {
            vertex_positions: start_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: start_normals,
            vertex_normal_indices: Some(vec![0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2]),
            armature_name: None,
        };

        let mut end_v0 = vec![0.0, 0.0, 0.0];
        let mut end_v1 = vec![1.0, 1.0, 1.0];
        let mut end_v2 = vec![2.0, 2.0, 2.0];
        let mut end_v3 = vec![3.0, 3.0, 3.0];
        let mut end_v4 = vec![0.0, 0.0, 0.0];
        let mut end_v5 = vec![1.0, 1.0, 1.0];
        let mut end_v6 = vec![2.0, 2.0, 2.0];
        let mut end_v7 = vec![3.0, 3.0, 3.0];

        let mut end_n0 = vec![4.0, 4.0, 4.0];
        let mut end_n1 = vec![5.0, 5.0, 5.0];
        let mut end_n2 = vec![6.0, 6.0, 6.0];

        let mut end_positions = vec![];
        end_positions.append(&mut end_v0);
        end_positions.append(&mut end_v1);
        end_positions.append(&mut end_v2);
        end_positions.append(&mut end_v3);
        end_positions.append(&mut end_v4);
        end_positions.append(&mut end_v5);
        end_positions.append(&mut end_v6);
        end_positions.append(&mut end_v7);

        let mut end_normals = vec![];
        end_normals.append(&mut end_n0);
        end_normals.append(&mut end_n1);
        end_normals.append(&mut end_n0);
        end_normals.append(&mut end_n1);
        end_normals.append(&mut end_n2);
        end_normals.append(&mut end_n2);
        end_normals.append(&mut end_n2);
        end_normals.append(&mut end_n2);

        let expected_mesh = BlenderMesh {
            vertex_positions: end_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: end_normals,
            vertex_normal_indices: None,
            armature_name: None,
        };

        mesh_to_combine.combine_vertex_indices();
        let combined_mesh = mesh_to_combine;

        assert_eq!(combined_mesh, expected_mesh);
    }
}
