/// Data for an individual vertex attribute such as positions, normals or uvs.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct VertexAttribute {
    /// The underlying vector of floats for this data
    data: Vec<f32>,
    /// Positions and normals have a size of 3 (x, y, z)
    /// Uvs have a size of 2 (u, v)
    size: AttributeSize,
}

/// The number of components per vertex.
///
/// For example - a position would have 3 components per vertex, a uv would have 2.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AttributeSize {
    Two = 2,
    Three = 3,
}

impl Default for AttributeSize {
    fn default() -> Self {
        AttributeSize::Three
    }
}

/// Used for vertex skinning
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BoneAttributes {
    bone_indices: VertexAttribute,
    bone_weights: VertexAttribute,
}

impl VertexAttribute {
    /// Create a new vertex attribute
    pub fn new(data: Vec<f32>, size: AttributeSize) -> VertexAttribute {
        VertexAttribute { data, size }
    }

    /// Get the underlying data for this attribute.
    /// Useful for buffering vertex data onto the GPU
    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

impl VertexAttribute {
    /// Set vertex data for an attribute with 3 components per vertex
    pub(crate) fn set_three_components(&mut self, idx: usize, comp1: f32, comp2: f32, comp3: f32) {
        match self.size {
            AttributeSize::Three => {
                self.data[idx * 3] = comp1;
                self.data[idx * 3 + 1] = comp2;
                self.data[idx * 3 + 2] = comp3;
            }
            _ => panic!("Does not have exactly three components"),
        };
    }

    /// Increment vertex data for an attribute with 3 components per vertex
    ///
    /// Useful for adding together two tangent vectors when preparing tangent data.
    pub(crate) fn increment_three_components(
        &mut self,
        idx: usize,
        comp1: f32,
        comp2: f32,
        comp3: f32,
    ) {
        match self.size {
            AttributeSize::Three => {
                self.data[idx * 3] += comp1;
                self.data[idx * 3 + 1] += comp2;
                self.data[idx * 3 + 2] += comp3;
            }
            _ => panic!("Does not have exactly three components"),
        };
    }

    /// Set vertex data for an attribute with 2 components per vertex
    pub(crate) fn set_two_components(&mut self, idx: usize, comp1: f32, comp2: f32) {
        match self.size {
            AttributeSize::Two => {
                self.data[idx * 2] = comp1;
                self.data[idx * 2 + 1] = comp2;
            }
            _ => panic!("Does not have exactly two components"),
        };
    }

    /// Push data to the end of the underlying vector
    pub(crate) fn push(&mut self, val: f32) {
        self.data.push(val)
    }
}
