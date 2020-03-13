use crate::vertex_attributes::MultiIndexedVertexAttributes;
use crate::BlenderMesh;

/// Indicates an error while calculating the tangents for a mesh's verticies
#[derive(Debug, Fail)]
pub enum TangentError {
    #[fail(display = "Cannot calculate vertex tangents for a mesh with no uvs")]
    NoVertexUvs,
}

impl BlenderMesh {
    /// Calculate the tangent for each face in the mesh - useful for normal mapping where you'll
    /// typically want to do lighting calculations in tangent space.
    ///
    /// We look at the first, second and third vertex position and uv coordinate in the
    /// face in order to calculate the tangent.
    ///
    /// This is useful for normal mapping.
    ///
    /// We'll push tangents to `.face_tangents`
    ///
    /// Then later in when combining indices we'll use these `.face_tangents` in order to
    /// generate `per_vertex_tangents`
    pub(crate) fn calculate_face_tangents(&self) -> Result<Vec<f32>, TangentError> {
        let multi = &self.multi_indexed_vertex_attributes;

        let MultiIndexedVertexAttributes {
            vertices_in_each_face,
            positions,
            uvs,
            ..
        } = multi;

        if uvs.is_none() {
            return Err(TangentError::NoVertexUvs)?;
        }
        let uvs = uvs.as_ref().unwrap();

        let mut total_indices_processed = 0;

        let mut face_tangents = vec![];

        // Iterate over each face and calculate the tangent for that face.
        for vertices_in_face in vertices_in_each_face.iter() {
            let vertices_in_face = *vertices_in_face;

            let idx = total_indices_processed as usize;

            // Get the first three vertex indices for this face
            let pos_idx_0 = positions.indices[idx];
            let pos_idx_1 = positions.indices[idx + 1];
            let pos_idx_2 = positions.indices[idx + 2];

            // Get the three UV indices for this face
            let uv_idx_0 = uvs.indices[idx];
            let uv_idx_1 = uvs.indices[idx + 1];
            let uv_idx_2 = uvs.indices[idx + 2];

            let pos0 = positions.attribute.data_at_idx(pos_idx_0);
            let pos1 = positions.attribute.data_at_idx(pos_idx_1);
            let pos2 = positions.attribute.data_at_idx(pos_idx_2);

            let uv0 = uvs.attribute.data_at_idx(uv_idx_0);
            let uv1 = uvs.attribute.data_at_idx(uv_idx_1);
            let uv2 = uvs.attribute.data_at_idx(uv_idx_2);

            let edge1 = (pos1[0] - pos0[0], pos1[1] - pos0[1], pos1[2] - pos0[2]);
            let edge2 = (pos2[0] - pos1[0], pos2[1] - pos1[1], pos2[2] - pos1[2]);

            let delta_uv1 = (uv1[0] - uv0[0], uv1[1] - uv0[1]);
            let delta_uv2 = (uv2[0] - uv1[0], uv2[1] - uv1[1]);

            let f = 1.0 / ((delta_uv1.0 * delta_uv2.1) - (delta_uv2.0 * delta_uv1.1));

            let tangent_x = f * ((delta_uv2.1 * edge1.0) - (delta_uv1.1 * edge2.0));
            let tangent_y = f * ((delta_uv2.1 * edge1.1) - (delta_uv1.1 * edge2.1));
            let tangent_z = f * ((delta_uv2.1 * edge1.2) - (delta_uv1.1 * edge2.2));

            face_tangents.push(tangent_x);
            face_tangents.push(tangent_y);
            face_tangents.push(tangent_z);

            total_indices_processed += vertices_in_face as u16;
        }

        Ok(face_tangents)
    }
}

/// Given a face idx, get the corresponding tangent
pub(crate) fn face_tangent_at_idx(face_tangents: &[f32], face_idx: usize) -> (f32, f32, f32) {
    (
        face_tangents[face_idx * 3],
        face_tangents[face_idx * 3 + 1],
        face_tangents[face_idx * 3 + 2],
    )
}

// Numbers in these tests were not verified by hand.
// Instead, we took this common tangent calculation formula wrote tests, and verified
// that the rendered models looked visually correct (meaning that our test values are also correct).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::concat_vecs;
    use crate::test_utils::*;

    /// Ensure that a mesh with no uvs returns TangentError::NoVertexUvs
    #[test]
    fn no_vertex_uvs() {
        let mesh: BlenderMesh = BlenderMesh::default();

        match mesh.calculate_face_tangents() {
            Ok(_) => unreachable!(),
            Err(TangentError::NoVertexUvs) => {}
        }
    }

    /// Properly calculates tangents for a mesh that has one triangle
    #[test]
    fn calculate_tangents_1_triangle() {
        let mesh: BlenderMesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: (
                    vec![0, 1, 2],
                    (
                        concat_vecs!(v(0), vec![1.0, 0.0, 0.0], vec![1.0, 1.0, 0.0]),
                        3,
                    )
                        .into(),
                )
                    .into(),
                uvs: Some(
                    (
                        vec![0, 1, 2],
                        (concat_vecs!(v2(0), vec![0.5, 0.0], v2(1)), 2).into(),
                    )
                        .into(),
                ),
                vertices_in_each_face: vec![3],
                ..MultiIndexedVertexAttributes::default()
            },
            ..BlenderMesh::default()
        };

        mesh.calculate_face_tangents().unwrap();

        assert_eq!(
            &mesh.calculate_face_tangents().unwrap(),
            // One face (a triangle) so only one face tangent vector
            &vec![2., 0., 0.]
        );
    }

    #[test]
    fn calculate_tangents_2_triangle() {
        let mut mesh: BlenderMesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: (
                    vec![0, 1, 2, 0, 2, 3],
                    (
                        concat_vecs!(
                            v(0),
                            vec![1.0, 0.0, 0.0],
                            vec![1.0, 1.0, 0.0],
                            vec![0., 1., 0.]
                        ),
                        3,
                    )
                        .into(),
                )
                    .into(),
                uvs: Some(
                    (
                        vec![0, 1, 2, 0, 2, 3],
                        (concat_vecs!(v2(0), vec![0.5, 0.0], v2(1), vec![0., 1.]), 2).into(),
                    )
                        .into(),
                ),
                vertices_in_each_face: vec![3, 3],
                ..MultiIndexedVertexAttributes::default()
            },
            ..BlenderMesh::default()
        };

        mesh.calculate_face_tangents().unwrap();

        assert_eq!(
            &mesh.calculate_face_tangents().unwrap(),
            // Two faces (two triangles) so two tangent vectors
            &vec![2., 0., 0., 1., 0., 0.]
        );
    }

    #[test]
    fn calculate_tangents_1_quad() {
        let mut mesh: BlenderMesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: (
                    vec![0, 1, 2, 3],
                    (
                        concat_vecs!(
                            v(0),
                            vec![1.0, 0.0, 0.0],
                            vec![1.0, 1.0, 0.0],
                            vec![0., 1., 0.]
                        ),
                        3,
                    )
                        .into(),
                )
                    .into(),
                uvs: Some(
                    (
                        vec![0, 1, 2, 3],
                        (concat_vecs!(v2(0), vec![0.5, 0.0], v2(1), vec![0., 1.]), 2).into(),
                    )
                        .into(),
                ),
                vertices_in_each_face: vec![4],
                ..MultiIndexedVertexAttributes::default()
            },
            ..BlenderMesh::default()
        };

        mesh.calculate_face_tangents().unwrap();

        assert_eq!(
            &mesh.calculate_face_tangents().unwrap(),
            // Two faces (two triangles) so two tangent vectors
            &vec![2., 0., 0.]
        );
    }
}
