mod vertex_attribute;

pub use self::vertex_attribute::{BoneAttributes, VertexAttribute};
use crate::bone::BoneInfluencesPerVertex;

mod single_indexed;
pub use self::single_indexed::*;

/// Points to the data for each vertex.
///
/// Typically by the time a `MeshIr` has been prepared for rendering it will have every 3 indices
/// correspond to one face - but this is not a requirement.
///
/// You could - for example - have 4 indices correspond to a face.
///
/// These can also be mixed. When exporting from Blender, for example, you might have
/// `VertexIndices` that are grouped by 3 sometimes and 4 other times and even more other times -
/// all within the same vector.
///
/// You can decipher this by referencing the [`vertices_in_each_face`]
///
/// ## A visualization
///
/// So way we have `VertexIndices` that correspond to a `vertex_uvs: (Vec<f32>, 2)`.
///
/// Say our `VertexIndices` and `vertex_uvs` look like the following:
///
/// ```rust,no_run
/// let vertex_indices = vec![0, 1, 2, 0, 2, 3];
/// let vertex_positions = vec![
///     0., 0.,
///     1., 0.,
///     1., 1.,
///     0., 1.,
/// ];
/// ```
///
/// In this case every vertex index corresponds to 3 vertex position floats.
///
/// | Index | Uv         |
/// | ---   | ---        |
/// | 0     | &[0., 0.,] |
/// | 1     | &[1., 0.,] |
/// | 2     | &[1., 1.,] |
/// | 3     | &[0., 1.,] |
///
/// When referenced in conjunction with `vertices_in_each_face` you can determine the
/// positions of the vertices in each face.
///
/// For example, if `vertices_in_each_face: [3, 4, 3]`, then the first three position indices
/// give you the positions for the triangle for the first face, then the next 4 position indices
/// give you the quad for the next face, then the next three give you positions for the next
/// triangle.
///
/// [`vertices_in_each_face`]: struct.MultiIndexVertexData.html#method.vertices_in_each_face
pub type VertexIndices = Vec<u16>;

/// Per vertex data from the BlenderMesh.
///
/// When exporting from Blender there data is exported with multiple indices,
/// then after running `combine_vertex_indices` there will be one single index
/// for all of the vertex data.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum VertexAttributes {
    /// The data has multiple indices per vertex, such as one for position data, uvs and normals,
    /// etc.
    Multi(MultiIndexedVertexAttributes),
    /// The data has one single index per vertex
    Single(SingleIndexedVertexAttributes),
}

impl Default for VertexAttributes {
    fn default() -> Self {
        VertexAttributes::Multi(MultiIndexedVertexAttributes::default())
    }
}

/// Vertex data with multiple indices - not suited for OpenGL and other single index rendering
/// pipelines, but good for on disk storage as their is less data duplicated when there are
/// multiple indices.
///
/// TODO: A HashMap so that we can have arbitrary vertex attributes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MultiIndexedVertexAttributes {
    // The number of vertices that comprise each face of the mesh.
    //
    // For example, [3, 4, 4, 3, 3, 4] would mean that the first face has 3 vertices (triangle),
    // the next face has 4 (quad), then the next face has 4, etc.
    //
    // ## Example Use Cases
    //
    // - Triangulation, where faces with more than 3 vertices need to be split into triangles.
    //
    // - Calculating vertex tangents, where all vertices in the same face will have the same
    //   tangent.
    pub(crate) vertices_in_each_face: Vec<u8>,
    pub(crate) material_index: Vec<u16>,
    pub(crate) positions: IndexedAttribute,
    pub(crate) normals: Option<IndexedAttribute>,
    pub(crate) uvs: Option<IndexedAttribute>,
    pub(crate) bone_influences: Option<VertexBoneInfluences>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct IndexedAttribute {
    pub(crate) indices: VertexIndices,
    pub(crate) attribute: VertexAttribute<f32>,
}

#[allow(missing_docs)]
impl IndexedAttribute {
    pub fn new(indices: VertexIndices, attribute: VertexAttribute<f32>) -> Self {
        IndexedAttribute { indices, attribute }
    }
}

impl From<(VertexIndices, VertexAttribute<f32>)> for IndexedAttribute {
    fn from(v: (VertexIndices, VertexAttribute<f32>)) -> Self {
        Self {
            indices: v.0,
            attribute: v.1,
        }
    }
}

impl From<MultiIndexedVertexAttributes> for VertexAttributes {
    fn from(m: MultiIndexedVertexAttributes) -> Self {
        VertexAttributes::Multi(m)
    }
}
impl From<SingleIndexedVertexAttributes> for VertexAttributes {
    fn from(s: SingleIndexedVertexAttributes) -> Self {
        VertexAttributes::Single(s)
    }
}

/// The amount that each bone in the mesh's parent armature influences each vertex.
///
/// For example, if `bone_indices = [0, 1, 2, 2, 5]` and
/// `bone_weights = [0.2, 0.4, 0.2, 0.5, 0.5]` and `bones_per_vertex = [3, 2]` then
/// the first vertex is influenced by bone 0 by 0.2, bone 1 by 0.4 and bone 2 by 0.2.
///
/// Then the second vertex is influenced by bone 2 by 0.5 and bone 5 by 0.5
///
/// TODO: Remove this and use VertexAttribute with something like attribute_size: Varies(vec![])
/// this allows us to handle all attributes the same way.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VertexBoneInfluences {
    /// The number of bones that affect each vertex.
    ///
    /// Example: [3, 5, 2] would mean that the first vertex is influenced by 3 bones, second by
    /// 5, and third by 2
    pub(crate) bones_per_vertex: BoneInfluencesPerVertex,
    /// The indices of the bones that affect each vertex.
    pub(crate) bone_indices: Vec<u8>,
    /// The corresponding weights of each bone index
    pub(crate) bone_weights: Vec<f32>,
}
