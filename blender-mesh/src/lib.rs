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
#[cfg_attr(test, derive(Default))]
pub struct BlenderMesh {
    /// [v1x, v1y, v1z, v2x, v2y, v2z, ...]
    pub vertex_positions: Vec<f32>,
    /// The indices within vertex positions that make up each triangle in our mesh.
    /// Three vertex position indices correspond to one triangle
    /// [0, 1, 2, 0, 2, 3, ...]
    pub vertex_position_indices: Vec<u16>,
    /// TODO: enum.. if they're all equal we replace the MyEnum::PerVertex(u8) with MyEnum::Equal(4)
    pub num_vertices_in_each_face: Vec<u8>,
    pub vertex_normals: Vec<f32>,
    pub vertex_normal_indices: Option<Vec<u16>>,
    pub armature_name: Option<String>,
    /// TODO: When we move to single index triangulate and add new vertices give those vertices the same group indices / weights
    /// TODO: A function that trims this down to `n` weights and indices per vertex. Similar to our
    /// triangulate function
    /// TODO: Make sure that when we combine vertex indices we expand our group weights
    pub vertex_group_indices: Option<Vec<u8>>,
    pub vertex_group_weights: Option<Vec<f32>>,
    /// TODO: enum.. if they're all equal we replace the MyEnum::PerVertex(u8) with MyEnum::Equal(4)
    pub num_groups_for_each_vertex: Option<Vec<u8>>, // TODO: textures: HashMap<TextureNameString, {uvs, uv_indices}>
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
    ///
    /// FIXME: Wrote a test and threw code at the wall until it passed. Need to refactor
    /// this extensively! Any work on this before refactoring will not be worth the time
    pub fn combine_vertex_indices(&mut self) {
        type PosIndex = u16;
        type NormalIndex = u16;
        type UvIndex = Option<u16>;
        type EncounteredIndices = HashMap<(PosIndex, NormalIndex, UvIndex), PosIndex>;

        let mut largest_pos_index = *self.vertex_position_indices.iter().max().unwrap() as usize;

        let mut encountered_indices: EncounteredIndices = HashMap::new();
        let mut encountered_pos_indices = HashSet::new();

        let mut single_normals = vec![];
        let mut single_index_pos_indices = vec![];
        let mut single_positions = vec![];

        let mut single_vertex_group_indices = self.vertex_group_indices.clone();
        let mut single_vertex_group_weights = self.vertex_group_weights.clone();
        let mut single_groups_per_vertex = self.num_groups_for_each_vertex.clone();

        single_index_pos_indices.resize(self.vertex_position_indices.len(), 0);

        let mut total_previous = 0;
        let vert_group_map = match self.num_groups_for_each_vertex.as_ref() {
            Some(num_groups_per) => {
                let mut map = HashMap::new();

                for (index, num) in num_groups_per.iter().enumerate() {
                    map.insert(index, total_previous);
                    total_previous += num;
                }

                Some(map)
            }
            None => None
        };

        for (vert_num, pos_index) in self.vertex_position_indices.iter().enumerate() {
            let pos_index = *pos_index;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[vert_num];

            // FIXME: Don't reallocate every iteration... Only reallocate when necessary.
            // Also trim the vector when we're done
            single_positions.resize(largest_pos_index * 3 + 7, 0.0);
            single_normals.resize(largest_pos_index * 3 + 7, 0.0);

            let mut index_to_reuse = None;

            {
                index_to_reuse = encountered_indices
                    .get(&(pos_index, normal_index, None))
                    .cloned();
            }

            if index_to_reuse.is_some() || !encountered_pos_indices.contains(&pos_index) {
                let index_to_reuse = match index_to_reuse {
                    Some(i) => i,
                    None => pos_index,
                };

                // TODO: vert_num -> element_array_index
                // TODO: pos_index / index_to_reuse -> vertex_id / vertex_id_to_reuse
                single_index_pos_indices[vert_num] = index_to_reuse;

                // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num
                single_positions[index_to_reuse as usize * 3] =
                    self.vertex_positions[pos_index as usize * 3];
                single_positions[index_to_reuse as usize * 3 + 1] =
                    self.vertex_positions[pos_index as usize * 3 + 1];
                single_positions[index_to_reuse as usize * 3 + 2] =
                    self.vertex_positions[pos_index as usize * 3 + 2];

                single_normals[index_to_reuse as usize * 3] =
                    self.vertex_normals[normal_index as usize * 3];
                single_normals[index_to_reuse as usize * 3 + 1] =
                    self.vertex_normals[normal_index as usize * 3 + 1];
                single_normals[index_to_reuse as usize * 3 + 2] =
                    self.vertex_normals[normal_index as usize * 3 + 2];

                encountered_pos_indices.insert(pos_index);
                encountered_indices.insert((pos_index, normal_index, None), index_to_reuse);
            } else {
                largest_pos_index += 1;

                single_index_pos_indices[vert_num] = largest_pos_index as u16;

                single_positions[largest_pos_index * 3] =
                    self.vertex_positions[pos_index as usize * 3];
                single_positions[largest_pos_index * 3 + 1] =
                    self.vertex_positions[pos_index as usize * 3 + 1];
                single_positions[largest_pos_index * 3 + 2] =
                    self.vertex_positions[pos_index as usize * 3 + 2];

                single_normals[largest_pos_index as usize * 3] =
                    self.vertex_normals[normal_index as usize * 3];
                single_normals[largest_pos_index as usize * 3 + 1] =
                    self.vertex_normals[normal_index as usize * 3 + 1];
                single_normals[largest_pos_index as usize * 3 + 2] =
                    self.vertex_normals[normal_index as usize * 3 + 2];

                match self.num_groups_for_each_vertex.as_ref() {
                    Some(num_groups_for_each_vertex) => {
                        let pos_index = pos_index as usize;
                        let foo = *vert_group_map.as_ref().unwrap().get(&pos_index).unwrap() as usize;

                        let num_groups_for_this_vertex = num_groups_for_each_vertex[pos_index as usize];
                        single_groups_per_vertex.as_mut().unwrap().push(num_groups_for_this_vertex);

                        for i in 0..num_groups_for_this_vertex {
                            let weight = single_vertex_group_weights.as_ref().unwrap()[foo + i as usize];
                            single_vertex_group_weights.as_mut().unwrap().push(weight);

                            let index = single_vertex_group_indices.as_ref().unwrap()[foo + i as usize];
                            single_vertex_group_indices.as_mut().unwrap().push(index);

                        }
                    }
                    None => {}
                };

                encountered_indices.insert(
                    (pos_index as u16, normal_index, None),
                    largest_pos_index as u16,
                );
            }
        }

        self.vertex_position_indices = single_index_pos_indices;
        self.vertex_normals = single_normals;
        self.vertex_positions = single_positions;

        self.vertex_positions.resize(largest_pos_index * 3 + 3, 0.0);
        self.vertex_normals.resize(largest_pos_index * 3 + 3, 0.0);

        self.vertex_group_indices = single_vertex_group_indices;
        self.num_groups_for_each_vertex = single_groups_per_vertex;
        self.vertex_group_weights = single_vertex_group_weights;

        self.vertex_normal_indices = None;
    }
}

impl BlenderMesh {
    /// When exporting a mesh from Blender, faces will usually have 4 vertices (quad) but some
    /// faces might have 3 (triangle).
    ///
    /// We read `self.num_vertices_in_each_face` to check how
    /// many vertices each face has.
    ///
    /// If a face has 4 vertices we convert it into two triangles, each with 3 vertices.
    ///
    /// # Panics
    ///
    /// Panics if a face has more than 4 vertices. In the future we might support 5+ vertices,
    /// but I haven't run into that yet. Not even sure if Blender can have faces with 5 vertices..
    pub fn triangulate(&mut self) {
        let mut triangulated_position_indices = vec![];
        let mut triangulated_face_vertex_counts = vec![];

        let mut face_pointer = 0;

        for num_verts_in_face in self.num_vertices_in_each_face.iter() {
            match num_verts_in_face {
                &3 => {
                    triangulated_face_vertex_counts.push(3);

                    triangulated_position_indices.push(self.vertex_position_indices[face_pointer]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 1]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 2]);

                    face_pointer += 3;
                }
                &4 => {
                    triangulated_face_vertex_counts.push(3);
                    triangulated_face_vertex_counts.push(3);

                    triangulated_position_indices.push(self.vertex_position_indices[face_pointer]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 1]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 2]);
                    triangulated_position_indices.push(self.vertex_position_indices[face_pointer]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 2]);
                    triangulated_position_indices
                        .push(self.vertex_position_indices[face_pointer + 3]);

                    face_pointer += 4;
                }
                _ => {
                    panic!("blender-mesh currently only supports triangulating faces with 3 or 4 vertices");
                }
            }
        }

        self.vertex_position_indices = triangulated_position_indices;
        self.num_vertices_in_each_face = triangulated_face_vertex_counts;
    }
}

impl BlenderMesh {
    /// Blender meshes get exported with a Z up coordinate system.
    /// Here we flip our coordinate system to be y up
    ///
    /// @see https://gamedev.stackexchange.com/a/7932
    ///
    /// TODO: When we have bone data we'll need to change them to port change-mat4-coordinate-system
    /// into here.
    /// https://github.com/chinedufn/change-mat4-coordinate-system/blob/master/change-mat4-coordinate-system.js
    pub fn y_up(&mut self) {
        for vert_num in 0..(self.vertex_positions.len() / 3) {
            let y_index = vert_num * 3 + 1;
            let z_index = y_index + 1;

            let new_z = -self.vertex_positions[y_index];
            self.vertex_positions[y_index] = self.vertex_positions[z_index];
            self.vertex_positions[z_index] = new_z;

            let new_z = -self.vertex_normals[y_index];
            self.vertex_normals[y_index] = self.vertex_normals[z_index];
            self.vertex_normals[z_index] = new_z;
        }
    }
}

pub type MeshNamesToData = HashMap<String, BlenderMesh>;
pub type FilenamesToMeshes = HashMap<String, MeshNamesToData>;

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

    // Concatenate a series of vectors into one vector
    macro_rules! concat_vecs {
        ( $( $vec:expr),* ) => {
            {
                let mut concatenated_vec = Vec::new();
                $(
                    concatenated_vec.append(&mut $vec.clone());
                )*
                concatenated_vec
            }
        }
    }

    #[test]
    fn combine_pos_norm_uv_indices() {
        let mut start_positions = concat_vecs!(v(0), v(1), v(2), v(3));
        let mut start_normals = concat_vecs!(v(4), v(5), v(6));

        // TODO: Breadcrumb - add vertex group weights and indices into the
        // start mesh and verify that we end up with the proper valuse

        let mut mesh_to_combine = BlenderMesh {
            vertex_positions: start_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: start_normals,
            vertex_normal_indices: Some(vec![0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2]),
            num_groups_for_each_vertex: Some(vec![3, 2, 5, 1]),
            vertex_group_indices: Some(vec![0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11]),
            vertex_group_weights: Some(vec![
                0.05, 0.8, 0.15, 0.5, 0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999,
            ]),
            ..BlenderMesh::default()
        };

        let mut end_positions = concat_vecs!(v(0), v(1), v(2), v(3), v(0), v(1), v(2), v(3));
        let mut end_normals = concat_vecs!(v(4), v(5), v(4), v(5), v(6), v(6), v(6), v(6));

        let expected_mesh = BlenderMesh {
            vertex_positions: end_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: end_normals,
            num_groups_for_each_vertex: Some(vec![3, 2, 5, 1, 3, 2, 5, 1]),
            vertex_group_indices: Some(vec![
                0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11, 0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11
            ]),
            vertex_group_weights: Some(vec![
                0.05, 0.8, 0.15, 0.5, 0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999, 0.05, 0.8, 0.15, 0.5,
                0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999,
            ]),
            ..BlenderMesh::default()
        };

        mesh_to_combine.combine_vertex_indices();
        let combined_mesh = mesh_to_combine;

        assert_eq!(combined_mesh, expected_mesh);
    }

    // TODO: TDD a Method to normalize groups per vertex to all be the same number

    #[test]
    fn triangulate_faces() {
        let mut start_mesh = BlenderMesh {
            vertex_position_indices: vec![0, 1, 2, 3, 4, 5, 6, 7],
            num_vertices_in_each_face: vec![4, 4],
            ..BlenderMesh::default()
        };

        start_mesh.triangulate();
        let triangulated_mesh = start_mesh;

        let expected_mesh = BlenderMesh {
            vertex_position_indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            num_vertices_in_each_face: vec![3, 3, 3, 3],
            ..BlenderMesh::default()
        };

        assert_eq!(triangulated_mesh, expected_mesh);
    }

    #[test]
    fn z_up_to_y_up() {
        let mut start_mesh = BlenderMesh {
            vertex_positions: vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
            vertex_normals: vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
            ..BlenderMesh::default()
        };

        start_mesh.y_up();

        let y_up_mesh = start_mesh;
        let expected_mesh = BlenderMesh {
            vertex_positions: vec![0.0, 2.0, -1.0, 0.0, 2.0, -1.0],
            vertex_normals: vec![0.0, 2.0, -1.0, 0.0, 2.0, -1.0],
            ..BlenderMesh::default()
        };

        assert_eq!(y_up_mesh, expected_mesh);
    }

    // Create a 3 dimensional vector with all three values the same.
    // Useful for quickly generating some fake vertex data.
    // v(0.0) -> vec![0.0, 0.0, 0.0]
    fn v(val: u8) -> Vec<f32> {
        vec![val as f32, val as f32, val as f32]
    }
}
