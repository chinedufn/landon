use crate::BlenderMesh;

impl BlenderMesh {
    /// Get the tangent vector for each vertex
    pub fn vertex_tangents(&self) -> Option<&Vec<f32>> {
        self.vertex_tangents.as_ref()
    }
}

/// Indicates an error while calculating the tangents for a mesh's verticies
#[derive(Debug, Fail)]
pub enum TangentError {
    #[fail(display = "Cannot calculate vertex tangents for a mesh with no uvs")]
    NoVertexUvs,
}

impl BlenderMesh {
    /// Calculate the tangent for each vertex in the mesh.
    /// This is useful for normal mapping.
    pub fn calculate_tangents(&mut self) -> Result<(), TangentError> {
        if self.vertex_uvs.is_none() {
            return Err(TangentError::NoVertexUvs)?;
        }

        //        let mut vertex_tangents = vec![];

        let vertex_count = self.vertex_position_indices.len();

        for idx in 0..(vertex_count / 3) {
            // Get the three vertex indices for this triangle, one per vertex
            let pos_index_0 = self.vertex_position_indices[idx] as usize;
            let pos_index_1 = self.vertex_position_indices[idx + 1] as usize;
            let pos_index_2 = self.vertex_position_indices[idx + 2] as usize;

            // Get the three UV indices for this triangle, one per vertex
            let uv_index_0 = self.vertex_uv_indices.as_ref().unwrap()[idx] as usize;
            let uv_index_1 = self.vertex_uv_indices.as_ref().unwrap()[idx + 1] as usize;
            let uv_index_2 = self.vertex_uv_indices.as_ref().unwrap()[idx + 2] as usize;
        }

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concat_vecs;
    use crate::test_utils::*;

    /// Ensure that a mesh with no uvs returns TangentError::NoVertexUvs
    #[test]
    fn no_vertex_uvs() {
        let mut mesh: BlenderMesh = BlenderMesh::default();

        match mesh.calculate_tangents() {
            Ok(_) => unreachable!(),
            Err(TangentError::NoVertexUvs) => {}
        }
    }

    /// Properly calculates tangents for a mesh that has one triangle
    #[test]
    fn calculate_tangents_1_triangle() {
        let mut mesh: BlenderMesh = BlenderMesh {
            vertex_positions: concat_vecs!(v(5), v(6), v(7)),
            vertex_uvs: Some(concat_vecs!(v2(1), v2(2), v2(3))),
            ..BlenderMesh::default()
        };

        mesh.calculate_tangents();

        //        assert_eq!(mesh.vertex_tangents().unwrap(), &vec![])
    }
}
