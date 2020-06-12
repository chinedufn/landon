use crate::vertex_attributes::VertexAttribute;

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
    pub(crate) vertices: Vec<Vertex>,
}

/// A vertex within a mesh.
///
/// You'll typically buffer the Vertex's data onto the GPU interleaved into a single buffer, and
/// then index into that buffer using the indices from [`SingleIndexedVertexAttributes`].
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) normal: Option<[f32; 3]>,
    pub(crate) face_tangent: Option<[f32; 3]>,
    pub(crate) uv: Option<[f32; 2]>,
    pub(crate) bones: Option<[BoneInfluence; 4]>,
}

impl Vertex {
    /// The model space position of this Vertex
    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    /// The surface normal for the face that this Vertex belongs to
    pub fn normal(&self) -> Option<[f32; 3]> {
        self.normal
    }

    /// The face tangent for the face that this Vertex belongs to
    pub fn face_tangent(&self) -> Option<[f32; 3]> {
        self.face_tangent
    }

    /// The UV coordinates for this Vertex
    pub fn uv(&self) -> Option<[f32; 2]> {
        self.uv
    }

    /// The bones that influence this Vertex.
    ///
    /// Currently a maximum of 4 bones is supported for no other reason than it being uncommon to
    /// need more than that.
    ///
    /// If this doesn't meet your needs pleas open an issue.
    ///
    /// If there are fewer than 4 influencing bones then the extra fake bones in this array will
    /// have weights of zero.
    pub fn bones(&self) -> Option<[BoneInfluence; 4]> {
        self.bones
    }
}

/// The index of a bone that influences the vertex along with the weighting of that influence
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneInfluence {
    pub(crate) bone_idx: u8,
    pub(crate) weight: f32,
}

impl BoneInfluence {
    /// The index of this bone within the mesh's parent armature's bones.
    pub fn bone_idx(&self) -> u8 {
        self.bone_idx
    }

    /// The amount between \[0.0, 1.0\] that this bone should influence the Vertex
    pub fn weight(&self) -> f32 {
        self.weight
    }
}

impl SingleIndexedVertexAttributes {
    /// For `SingleIndexVertexData` every 3 indices corresponds to one triangle.
    ///
    /// There can not be any other faces (quads, ngons) - only triangles.
    pub fn indices(&self) -> &Vec<u16> {
        &self.indices
    }

    /// All of the vertex data for the mesh.
    ///
    /// You can index into this data using [`SingleIndexedVertexAttributes#method.indices`]
    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub(crate) fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }
}
