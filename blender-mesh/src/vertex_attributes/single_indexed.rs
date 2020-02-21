use crate::vertex_attributes::{BoneAttributes, VertexAttribute};

mod interleave;

pub use self::interleave::*;

/// Most 3D model file formats export vertex data with multiple indices.
///
/// There might be indices for the positions, normals and uvs.
///
/// The `SingleIndexVertexData` is vertex data that only has one index.
///
/// When we've run [`BlenderMesh.combine_vertex_indices`] we'll end up generating
/// `SingleIndexVertexData`
///
/// [`BlenderMesh.combine_vertex_indices`]: ../struct.BlenderMesh.html#method.combine_vertex_indices
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SingleIndexedVertexAttributes {
    pub(crate) indices: Vec<u16>,
    pub(crate) positions: VertexAttribute<f32>,
    pub(crate) normals: Option<VertexAttribute<f32>>,
    pub(crate) face_tangents: Option<VertexAttribute<f32>>,
    pub(crate) uvs: Option<VertexAttribute<f32>>,
    pub(crate) bones: Option<BoneAttributes>,
}

impl SingleIndexedVertexAttributes {
    /// For `SingleIndexVertexData` every 3 indices corresponds to one triangle.
    ///
    /// There can not be any other faces (quads, ngons) - only triangles.
    pub fn indices(&self) -> &Vec<u16> {
        &self.indices
    }

    /// Every 3 floats corresponds to one vertex's position
    pub fn positions(&self) -> &VertexAttribute<f32> {
        &self.positions
    }

    /// Every 3 floats corresponds to one vertex's normal.
    pub fn normals(&self) -> Option<&VertexAttribute<f32>> {
        self.normals.as_ref()
    }

    /// Every 3 floats corresponds to one tangent vector - useful for normal mapping.
    pub fn face_tangents(&self) -> Option<&VertexAttribute<f32>> {
        self.face_tangents.as_ref()
    }

    /// Every 2 floats corresponds to one vertex's uv.
    pub fn uvs(&self) -> Option<&VertexAttribute<f32>> {
        self.uvs.as_ref()
    }

    /// The indices of the joints that influence each bone.
    ///
    /// The number of floats per vertex can vary and can be found using
    /// [`VertexAttribute.attribute_size()`].
    ///
    /// [`VertexAttribute.attribute_size()`]: struct.VertexAttribute.html#method.attribute_size
    pub fn bones_influencers(&self) -> Option<&VertexAttribute<u8>> {
        Some(&self.bones.as_ref()?.bone_influencers)
    }

    /// The weights of each bone influencer.
    ///
    /// The number of floats per vertex can vary and can be found using
    /// [`VertexAttribute.attribute_size()`].
    ///
    /// [`VertexAttribute.attribute_size()`]: struct.VertexAttribute.html#method.attribute_size
    pub fn bone_influencer_weights(&self) -> Option<&VertexAttribute<f32>> {
        Some(&self.bones.as_ref()?.bone_weights)
    }
}
