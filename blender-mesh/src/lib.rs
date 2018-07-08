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
use std::cmp::max;
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
    ///  1) Easier because we we can unit test that here vs. a blender python script that's much
    ///     trickier to test.
    ///  2) Reduces amount of data required to represent the model on disk.
    ///
    /// OpenGL only supports one index buffer, we convert our vertex data
    /// from having three indices to having one. This usually requires some duplication of
    /// vertex data. We duplicate the minimum amount of vertex data necessary.
    ///
    /// FIXME: Wrote a test and threw code at the wall until it passed. Need to refactor
    /// this extensively! Any work on this before refactoring will not be worth the time
    /// Split this up into smaller functions that it calls, and clean up those functions.
    pub fn combine_vertex_indices(&mut self) {
        type PosIndex = u16;
        type NormalIndex = u16;
        type UvIndex = Option<u16>;
        type EncounteredIndices = HashMap<(PosIndex, NormalIndex, UvIndex), PosIndex>;

        let mut largest_vert_id = *self.vertex_position_indices.iter().max().unwrap() as usize;

        let mut encountered_vert_data: EncounteredIndices = HashMap::new();
        let mut encountered_vert_ids = HashSet::new();

        let mut single_normals = vec![];
        single_normals.resize(
            max(self.vertex_positions.len(), self.vertex_normals.len()),
            0.0,
        );

        let mut new_pos_indices = vec![];

        let mut new_group_indices = self.vertex_group_indices.clone();
        let mut new_group_weights = self.vertex_group_weights.clone();
        let mut new_groups_for_each_vert = self.num_groups_for_each_vertex.clone();

        new_pos_indices.resize(self.vertex_position_indices.len(), 0);

        let mut total_previous: u32 = 0;
        let vert_group_map = match self.num_groups_for_each_vertex.as_ref() {
            Some(num_groups_per) => {
                let mut map = HashMap::new();

                for (index, num) in num_groups_per.iter().enumerate() {
                    map.insert(index, total_previous);
                    total_previous += *num as u32;
                }

                Some(map)
            }
            None => None,
        };

        for (elem_array_index, vert_id) in self.vertex_position_indices.iter().enumerate() {
            let vert_id = *vert_id;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[elem_array_index];

            let mut vert_id_to_reuse = encountered_vert_data
                .get(&(vert_id, normal_index, None))
                .cloned();

            if vert_id_to_reuse.is_some() || !encountered_vert_ids.contains(&vert_id) {
                let vert_id = match vert_id_to_reuse {
                    Some(i) => i,
                    None => vert_id,
                };

                // TODO: vert_num -> element_array_index
                // TODO: pos_index / index_to_reuse -> vertex_id / vertex_id_to_reuse
                new_pos_indices[elem_array_index] = vert_id;

                // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num

                single_normals[vert_id as usize * 3] = self.vertex_x_normal(normal_index);
                single_normals[vert_id as usize * 3 + 1] = self.vertex_y_normal(normal_index);
                single_normals[vert_id as usize * 3 + 2] = self.vertex_z_normal(normal_index);

                encountered_vert_ids.insert(vert_id);
                encountered_vert_data.insert((vert_id, normal_index, None), vert_id);
            } else {
                largest_vert_id += 1;

                new_pos_indices[elem_array_index] = largest_vert_id as u16;

                let x = self.vertex_x_pos(vert_id);
                self.vertex_positions.push(x);

                let y = self.vertex_y_pos(vert_id);
                self.vertex_positions.push(y);

                let z = self.vertex_z_pos(vert_id);
                self.vertex_positions.push(z);

                single_normals.push(self.vertex_x_normal(normal_index));
                single_normals.push(self.vertex_y_normal(normal_index));
                single_normals.push(self.vertex_z_normal(normal_index));

                match self.num_groups_for_each_vertex.as_ref() {
                    Some(num_groups_for_each_vertex) => {
                        let pos_index = vert_id as usize;
                        let foo =
                            *vert_group_map.as_ref().unwrap().get(&pos_index).unwrap() as usize;

                        let num_groups_for_this_vertex =
                            num_groups_for_each_vertex[pos_index as usize];
                        new_groups_for_each_vert
                            .as_mut()
                            .unwrap()
                            .push(num_groups_for_this_vertex);

                        for i in 0..num_groups_for_this_vertex {
                            let weight = new_group_weights.as_ref().unwrap()[foo + i as usize];
                            new_group_weights.as_mut().unwrap().push(weight);

                            let index = new_group_indices.as_ref().unwrap()[foo + i as usize];
                            new_group_indices.as_mut().unwrap().push(index);
                        }
                    }
                    None => {}
                };

                encountered_vert_data
                    .insert((vert_id as u16, normal_index, None), largest_vert_id as u16);
            }
        }

        self.vertex_position_indices = new_pos_indices;
        self.vertex_normals = single_normals;

        self.vertex_positions.resize(largest_vert_id * 3 + 3, 0.0);
        self.vertex_normals.resize(largest_vert_id * 3 + 3, 0.0);

        if self.armature_name.is_some() {
            self.vertex_group_indices = new_group_indices;
            self.num_groups_for_each_vertex = new_groups_for_each_vert;
            self.vertex_group_weights = new_group_weights;
        }

        self.vertex_normal_indices = None;
    }

    fn vertex_x_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 0]
    }
    fn vertex_y_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 1]
    }
    fn vertex_z_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 2]
    }
    fn vertex_x_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 0]
    }
    fn vertex_y_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 1]
    }
    fn vertex_z_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 2]
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

impl BlenderMesh {
    /// Different vertices might have different numbers of bones that influence them.
    /// A vertex near the shoulder might be influenced by the neck and upper arm and sternum,
    /// while a vertex in a toe might only be influenced by a toe bone.
    ///
    /// When passing data to the GPU, each vertex needs the same number of bone attributes, so
    /// we must add/remove bones from each vertex to get them equal.
    ///
    /// Say we're setting 3 groups per vertex:
    ///  - If a vertex has one vertex group (bone) we will create two fake bones with 0.0 weight.
    ///  - If a vertex has 5 bones we'll remove the one with the smallest weighting (influence).
    pub fn set_groups_per_vertex(&mut self, count: u8) {
        let mut normalized_group_indices = vec![];
        let mut normalized_group_weights = vec![];

        let mut current_index: u32 = 0;

        {
            let mut indices = self.vertex_group_indices.as_mut().unwrap();
            let mut weights = self.vertex_group_weights.as_mut().unwrap();

            self.num_groups_for_each_vertex = Some(
                self.num_groups_for_each_vertex
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|group_count| {
                        let mut vertex_indices = vec![];
                        let mut vertex_weights = vec![];

                        for index in current_index..(current_index + *group_count as u32) {
                            vertex_indices.push(index);
                            vertex_weights.push(weights[index as usize]);
                        }

                        vertex_weights.sort_by(|a, b| b.partial_cmp(a).unwrap());
                        vertex_indices.sort_by(|a, b| {
                            weights[*b as usize]
                                .partial_cmp(&weights[*a as usize])
                                .unwrap()
                        });

                        let mut vertex_indices: Vec<u8> = vertex_indices
                            .iter()
                            .map(|i| indices[*i as usize])
                            .collect();

                        vertex_indices.resize(count as usize, 0);
                        vertex_weights.resize(count as usize, 0.0);

                        normalized_group_indices.append(&mut vertex_indices);
                        normalized_group_weights.append(&mut vertex_weights);

                        current_index += *group_count as u32;
                        count
                    })
                    .collect(),
            );
        }

        self.vertex_group_indices = Some(normalized_group_indices);
        self.vertex_group_weights = Some(normalized_group_weights);
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
        let start_positions = concat_vecs!(v(0), v(1), v(2), v(3));
        let start_normals = concat_vecs!(v(4), v(5), v(6));

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

        let end_positions = concat_vecs!(v(0), v(1), v(2), v(3), v(0), v(1), v(2), v(3));
        let end_normals = concat_vecs!(v(4), v(5), v(4), v(5), v(6), v(6), v(6), v(6));

        let expected_mesh = BlenderMesh {
            vertex_positions: end_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: end_normals,
            num_groups_for_each_vertex: Some(vec![3, 2, 5, 1, 3, 2, 5, 1]),
            vertex_group_indices: Some(vec![
                0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11, 0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11,
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

    #[test]
    fn set_joints_per_vert() {
        let mut start_mesh = BlenderMesh {
            vertex_group_indices: Some(vec![0, 2, 3, 4, 0, 1, 3, 2]),
            num_groups_for_each_vertex: Some(vec![1, 3, 4]),
            vertex_group_weights: Some(vec![1.0, 0.5, 0.2, 0.3, 0.6, 0.15, 0.1, 0.15]),
            ..BlenderMesh::default()
        };

        start_mesh.set_groups_per_vertex(3);
        let three_joints_per_vert = start_mesh;

        let expected_mesh = BlenderMesh {
            vertex_group_indices: Some(vec![0, 0, 0, 2, 4, 3, 0, 1, 2]),
            num_groups_for_each_vertex: Some(vec![3, 3, 3]),
            vertex_group_weights: Some(vec![1.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.6, 0.15, 0.15]),
            ..BlenderMesh::default()
        };

        assert_eq!(three_joints_per_vert, expected_mesh);
    }

    // Create a 3 dimensional vector with all three values the same.
    // Useful for quickly generating some fake vertex data.
    // v(0.0) -> vec![0.0, 0.0, 0.0]
    fn v(val: u8) -> Vec<f32> {
        vec![val as f32, val as f32, val as f32]
    }
}
