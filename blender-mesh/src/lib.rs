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

pub use self::combine_indices::CreateSingleIndexConfig;
pub use self::export::*;
use crate::bone::BoneInfluencesPerVertex;
use crate::bounding_box::BoundingBox;
use crate::material::PrincipledBSDF;
use crate::vertex_data::{VertexAttribute, VertexData};
pub use material::{Channel, MaterialInput};
use serde_json;
use serde_json::Error;
use std::collections::HashMap;

mod bone;
mod bounding_box;
mod combine_indices;
mod export;
mod individual_vertex;
mod material;
mod tangent;
mod triangulate;
mod vertex_data;
mod y_up;

#[cfg(test)]
mod test_utils;

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
    bone_influences_per_vertex: Option<BoneInfluencesPerVertex>,
    pub bounding_box: BoundingBox,
    /// A map of material name (in Blender) to the material's data
    materials: HashMap<String, PrincipledBSDF>,
    /// Tangent vectors per vertex, useful for normal mapping.
    ///
    /// These get set during [`BlenderMesh.combine_indices`], if there are triangle_tangents.
    ///
    /// Useful for normal mapping.
    per_vertex_tangents: Option<VertexAttribute>,
    /// Tangent vector to the vertex, calculated using [`BlenderMesh.calculate_face_tangents`].
    ///
    /// [`BlenderMesh.calculate_face_tangents`]: struct.BlenderMesh.html#method.calculate_face_tangents
    face_tangents: Option<Vec<f32>>,
    #[serde(default)] // TODO: Temporary until we move all of the data above into VertexData
    vertex_data: VertexData,
}

impl BlenderMesh {
    // TODO: Delete this.. let the consumer worry about serializing / deserializing
    pub fn from_json(json_str: &str) -> Result<BlenderMesh, Error> {
        serde_json::from_str(json_str)
    }
}

/// Concatenate a series of vectors into one vector.
///
/// Useful for generating fake vertex data for unit tests.
///
/// ```ignore
/// assert_eq!(
///     concat_vecs!(vec![1, 2, 3], vec![4,5]),
///     vec![1, 2, 3, 4, 5]
/// );
/// ```
#[cfg(test)]
#[macro_export]
#[cfg(test)]
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
