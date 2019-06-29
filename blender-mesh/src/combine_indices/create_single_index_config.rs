/// Configuration for combining multiple indices into a single index
#[derive(Debug, Default)]
pub struct CreateSingleIndexConfig {
    /// The number of bones that influence each vertex.
    ///
    /// If unset then no bone influences will be included in the final single indexed VertexData.
    pub bone_influences_per_vertex: Option<u8>,
}
