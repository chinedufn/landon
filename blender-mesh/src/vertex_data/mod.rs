mod vertex_attribute;

pub(crate) use self::vertex_attribute::{AttributeSize, BoneAttributes, VertexAttribute};

/// Per vertex data from the BlenderMesh.
///
/// When exporting from Blender there data is exported with multiple indices,
/// then after running `combine_vertex_indices` there will be one single index
/// for all of the vertex data.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum VertexData {
    /// The data has multiple indices per vertex, such as one for position data, uvs and normals,
    /// etc.
    Multi(MultiIndexVertexData),
    /// The data has one single index per vertex
    Single(SingleIndexVertexData),
}

impl Default for VertexData {
    fn default() -> Self {
        VertexData::Multi(MultiIndexVertexData::default())
    }
}

/// Vertex data with multiple indices - not suited for OpenGL and other single index rendering
/// pipelines, but good for on disk storage as their is less data duplicated when there are
/// multiple indices.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct MultiIndexVertexData {
    /// The number of vertices that comprise each face of the mesh.
    ///
    /// For example, [3, 4, 4, 3, 3, 4] would mean that the first face has 3 vertices (triangle),
    /// the next face has 4 (quad), then the next face has 4, etc.
    ///
    /// ## Example Use Cases
    ///
    /// - Triangulation, where faces with more than 3 vertices need to be split into triangles.
    ///
    /// - Calculating vertex tangents, where all vertices in the same face will have the same
    ///   tangent.
    vertices_in_each_face: Vec<u8>,
    /// The amount that the bones in the arent armature influence each vertex
    bone_influences: Option<BoneInfluences>,
    /// All of the x, y and z positions of the vertices in this mesh, indexed by `position_indices`.
    ///
    /// For example, [0., 1., 2., 3., 4. 5.] would mean that there are two vertex positions.
    /// (0., 1., 2.) and (3., 4., 5.).
    ///
    /// This does not, however, mean that there are two vertices. There could be multiple vertices
    /// that happened to have the same positions.
    positions: VertexAttribute,
    /// Indices of each vertex positions for this mesh. Position index 0 would be the 0 index
    /// in the positions vector.
    ///
    /// Position index 1 would be the [3] index in the positions vector. index 2 -> [6], etc.
    ///
    /// When referenced in conjunction with `vertices_in_each_face` you can determine the
    /// positions of the vertices in each face.
    ///
    /// For example, if `vertices_in_each_face: [3, 4, 3]`, then the first three position indices
    /// give you the positions for the triangle for the first face, then the next 4 position indices
    /// give you the quad for the next face, then the next three give you positions for the next
    /// triangle.
    position_indices: Vec<u16>,
    /// All of the normals, indexed by `normal_indices`
    normals: VertexAttribute,
    /// Indices into the normals
    normal_indices: Vec<u16>,
    /// All of the uvs, indexed by uv_indices
    uvs: Option<VertexAttribute>,
    /// Indices into the uvs
    uv_indices: Option<Vec<u16>>,
}

/// Most 3D model file formats export vertex data with multiple indices.
///
/// There might be indices for the positions, normals and uvs.
///
/// The `SingleIndexVertexData` is vertex data that only has one index.
///
/// When we've run [`BlenderMesh.combine_vertex_indices`] we'll end up generating
/// `SingleIndexVertexData`
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SingleIndexVertexData {
    positions: VertexAttribute,
    normals: VertexAttribute,
    uvs: Option<VertexAttribute>,
    bones: Option<BoneAttributes>,
}

/// The amount that each bone in the mesh's parent armature influences each vertex.
///
/// For example, if `bone_indices = [0, 1, 2, 2, 5]` and
/// `bone_weights = [0.2, 0.4, 0.2, 0.5, 0.5]` and `bones_per_vertex = [3, 2]` then
/// the first vertex is influenced by bone 0 by 0.2, bone 1 by 0.4 and bone 2 by 0.2.
///
/// Then the second vertex is influenced by bone 2 by 0.5 and bone 5 by 0.5
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BoneInfluences {
    /// The number of bones that affect each vertex.
    ///
    /// Example: [3, 5, 2] would mean that the first vertex is influenced by 3 bones, second by
    /// 5, and third by 2
    bones_per_vertex: Vec<u8>,
    /// The indices of the bones that affect each vertex.
    bone_indices: Vec<u8>,
    /// The corresponding weights of each bone index
    bone_weights: Vec<f32>,
}
