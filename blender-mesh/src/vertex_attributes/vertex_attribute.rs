use std::ops::Deref;

/// Data for an individual vertex attribute such as positions, normals or uvs.
///
/// All of the x, y and z positions of the vertices in this mesh, indexed by `position_indices`.
///
/// For example, vec![0., 10., 2., 65.2, 4., 5.] with an attribute size of three would mean that
/// there are is data for two vertices.
///
/// Data set one being (0., 10., 2.) and (65.2, 4., 5.).
///
/// This does not, however, mean that there are two vertices in the mesh that is using these
/// vertices.
///
/// There could be multiple vertices that happened to have the same positions.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct VertexAttribute<T> {
    pub(crate) data: Vec<T>,
    pub(crate) attribute_size: u8,
}

impl<T> Deref for VertexAttribute<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// TODO: Remove this - just quickly lightly refactoring the codebase ..
#[cfg(test)]
impl<T> From<(Vec<T>, u8)> for VertexAttribute<T> {
    fn from(v: (Vec<T>, u8)) -> Self {
        VertexAttribute {
            data: v.0,
            attribute_size: v.1,
        }
    }
}

impl<T> VertexAttribute<T> {
    /// TODO: Introduce thiserror and add error handling to this library
    pub fn new(data: Vec<T>, attribute_size: u8) -> Result<VertexAttribute<T>, ()> {
        if attribute_size as usize % data.len() != 0 {
            // Return an error ...
        }

        Ok(VertexAttribute {
            data,
            attribute_size,
        })
    }

    #[allow(missing_docs)]
    pub fn as_slice(&self) -> &[T] {
        &self.data[..]
    }

    /// The number of values per vertex.
    ///
    /// Typically positions and normals have a size of 3 (x, y, z)
    ///
    /// Uvs have a size of 2 (u, v)
    ///
    /// But other data types can vary. Bone influences / weights might have 3, 4, or some other
    /// number attribute size depending on the application's needs.
    pub fn attribute_size(&self) -> u8 {
        self.attribute_size
    }
}

/// Used for vertex skinning
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BoneAttributes {
    pub(crate) bone_influencers: VertexAttribute<u8>,
    pub(crate) bone_weights: VertexAttribute<f32>,
}

impl<T> VertexAttribute<T> {
    /// Get the underlying data for this attribute.
    /// Useful for buffering vertex data onto the GPU
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Get the underlying data for this attribute mutably.
    pub(crate) fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }
}

impl<T> VertexAttribute<T> {
    /// Given a vertex indexm return the data at that index.
    ///
    /// If there are 3 attributes per vertex the size will be 3, if 2 then 2, etc.
    pub(crate) fn data_at_idx(&self, vertex_idx: u16) -> &[T] {
        let attribute_size = self.attribute_size as usize;
        let idx = (vertex_idx as usize) * attribute_size;

        &self.data[idx..idx + attribute_size]
    }

    /// Set vertex data for an attribute with 3 components per vertex
    pub(crate) fn set_three_components(&mut self, idx: usize, comp1: T, comp2: T, comp3: T) {
        self.data[idx * 3] = comp1;
        self.data[idx * 3 + 1] = comp2;
        self.data[idx * 3 + 2] = comp3;
    }

    /// Increment vertex data for an attribute with 3 components per vertex
    ///
    /// Useful for adding together two tangent vectors when preparing tangent data.
    pub(crate) fn increment_three_components(&mut self, idx: usize, comp1: T, comp2: T, comp3: T)
    where
        T: std::ops::AddAssign,
    {
        self.data[idx * 3] += comp1;
        self.data[idx * 3 + 1] += comp2;
        self.data[idx * 3 + 2] += comp3;
    }

    /// Set vertex data for an attribute with 2 components per vertex
    pub(crate) fn set_two_components(&mut self, idx: usize, comp1: T, comp2: T) {
        self.data[idx * 2] = comp1;
        self.data[idx * 2 + 1] = comp2;
    }

    /// Push data to the end of the underlying vector
    pub(crate) fn push(&mut self, val: T) {
        self.data.push(val)
    }
}
