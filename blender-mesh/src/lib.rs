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
#[macro_use]
extern crate serde_derive;

pub use self::export::*;
use crate::bounding_box::BoundingBox;
use crate::material::PrincipledBSDF;
use serde_json;
use serde_json::Error;
use std::collections::HashMap;

mod bounding_box;
mod combine_indices;
mod export;
mod material;
mod y_up;
pub use material::{Channel, MaterialInput};

/// Something went wrong in the Blender child process that was trying to parse your mesh data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export meshes from a `.blend` file.
    #[fail(
        display = "There was an issue while exporting meshes: Blender stderr output: {}",
        _0
    )]
    Stderr(String),
}

/// All of the data about a Blender mesh
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
#[serde(deny_unknown_fields)]
pub struct BlenderMesh {
    /// All of the mesh's vertices. Three items in the vector make one vertex.
    /// So indices 0, 1 and 2 are a vertex, 3, 4 and 5 are a vertex.. etc.
    /// [v1x, v1y, v1z, v2x, v2y, v2z, ...]
    pub vertex_positions: Vec<f32>,
    /// The indices within vertex positions that make up each triangle in our mesh.
    /// Three vertex position indices correspond to one triangle
    /// [0, 1, 2, 0, 2, 3, ...]
    pub vertex_position_indices: Vec<u16>,
    /// TODO: enum..? if they're all equal we replace the MyEnum::PerVertex(Vec<u8>) with MyEnum::Equal(4)
    pub num_vertices_in_each_face: Vec<u8>,
    pub vertex_normals: Vec<f32>,
    pub vertex_normal_indices: Option<Vec<u16>>,
    /// If your mesh is textured these will be all of the mesh's vertices' uv coordinates.
    /// Every vertex has two UV coordinates.
    /// [v1s, v1t, v2s, v2t, v3s, v3t]
    /// TODO: Combine vertex_uvs, vertex_uv_indices, texture_name into texture_info
    pub vertex_uvs: Option<Vec<f32>>,
    pub vertex_uv_indices: Option<Vec<u16>>,
    pub armature_name: Option<String>,
    /// TODO: When we move to single index triangulate and add new vertices give those vertices the same group indices / weights
    /// TODO: A function that trims this down to `n` weights and indices per vertex. Similar to our
    /// triangulate function
    /// TODO: Make sure that when we combine vertex indices we expand our group weights
    pub vertex_group_indices: Option<Vec<u8>>,
    pub vertex_group_weights: Option<Vec<f32>>,
    /// TODO: enum..? if they're all equal we replace the MyEnum::PerVertex(Vec<u8>) with MyEnum::Equal(4)
    pub num_groups_for_each_vertex: Option<Vec<u8>>, // TODO: textures: HashMap<TextureNameString, {uvs, uv_indices}>,
    pub bounding_box: BoundingBox,
    /// A map of material name (in Blender) to the material's data
    pub(self) materials: HashMap<String, PrincipledBSDF>,
}

impl BlenderMesh {
    // TODO: Delete this.. let the consumer worry about serializing / deserializing
    pub fn from_json(json_str: &str) -> Result<BlenderMesh, Error> {
        serde_json::from_str(json_str)
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
            let indices = self.vertex_group_indices.as_mut().unwrap();
            let weights = self.vertex_group_weights.as_mut().unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
