use crate::SingleIndexedVertexAttributes;
use nalgebra::{Point3, Vector3};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// An error when blending normals
#[derive(Debug, thiserror::Error)]
pub enum WeightedNormalsError {
    #[error("There were no normals to weight")]
    NoNormals,
}

impl SingleIndexedVertexAttributes {
    /// Alter normals to be both
    ///   surface weighted (connected triangle size) and
    ///   angle weighted (angle of connected triangle corner)
    ///
    /// @see http://www.bytehazard.com/articles/vertnorm.html
    ///
    /// TODO: We could also implement this for multi-indexed - but we should wait until we
    /// refactor / replace the combine_indices function because, for example, if we weight normals
    /// before we calculate face tangents our face tangents will be incorrect.
    /// In general this entire crate needs to be heavily TDD"d and refactored into something clean..
    pub fn face_weight_normals(&mut self) -> Result<(), WeightedNormalsError> {
        let mut encountered_positions: HashMap<[u32; 3], SharedVertexPositionWeightedNormal> =
            HashMap::new();

        for (vertex_num, pos_norm_data_idx) in self.indices.iter().enumerate() {
            let normals = self.normals.as_ref().unwrap();

            let pos_norm_data_idx = *pos_norm_data_idx as usize;

            let pos = &self.positions.data[to_range(pos_norm_data_idx)];
            let pos_point = Point3::new(pos[0], pos[1], pos[2]);

            let face_normal = &normals.data[to_range(pos_norm_data_idx)];
            let face_normal = Vector3::new(face_normal[0], face_normal[1], face_normal[2]);

            let (connected_vertex_1, connected_vertex_2) = match vertex_num % 3 {
                0 => (vertex_num + 1, vertex_num + 2),
                1 => (vertex_num - 1, vertex_num + 1),
                2 => (vertex_num - 2, vertex_num - 1),
                _ => unreachable!(),
            };

            let connected_vertex_1 =
                &self.positions.data[to_range(self.indices[connected_vertex_1] as usize)];
            let connected_vertex_1 = Point3::new(
                connected_vertex_1[0],
                connected_vertex_1[1],
                connected_vertex_1[2],
            );

            let connected_vertex_2 =
                &self.positions.data[to_range(self.indices[connected_vertex_2] as usize)];
            let connected_vertex_2 = Point3::new(
                connected_vertex_2[0],
                connected_vertex_2[1],
                connected_vertex_2[2],
            );

            let weighted_normal = weight_normal_using_surface_and_angle(
                face_normal,
                connected_vertex_1 - pos_point.clone(),
                connected_vertex_2 - pos_point,
            );

            let pos_hash = [pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits()];
            match encountered_positions.entry(pos_hash) {
                Entry::Occupied(mut previous) => {
                    previous
                        .get_mut()
                        .normals_to_overwrite
                        .push(pos_norm_data_idx);
                    previous.get_mut().weighted_normal += weighted_normal;
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(SharedVertexPositionWeightedNormal {
                        normals_to_overwrite: vec![pos_norm_data_idx],
                        weighted_normal,
                    });
                }
            };
        }

        for (_pos_hash, overlapping_vertices) in encountered_positions.into_iter() {
            let weighted_normal = overlapping_vertices.weighted_normal.normalize();
            let weighted_normal = weighted_normal.as_slice();

            for normal_data_idx in overlapping_vertices.normals_to_overwrite {
                self.normals.as_mut().unwrap().data[to_range(normal_data_idx)]
                    .copy_from_slice(weighted_normal);
            }
        }

        Ok(())
    }
}

fn to_range(idx: usize) -> std::ops::Range<usize> {
    3 * idx..3 * idx + 3
}

fn connected_vert_nums(vert_num: usize) -> (usize, usize) {
    unimplemented!()
}

// While we iterate through our positions we keep track of which normals corresponded to
// duplicate positions - along with the weighted normal.
// Then when we're done we go back to all of the corresponding normals for each shared
// position and overwrite them with the new weighted normal.
#[derive(Debug)]
struct SharedVertexPositionWeightedNormal {
    normals_to_overwrite: Vec<usize>,
    weighted_normal: Vector3<f32>,
}

/// Alter normals to be both
///
///   surface weighted (area of the connected face) and
///   angle weighted (angle of the current vertex corner triangle corner)
///
/// We assume that the connected face is a triangle - and as such we only need the two vectors
/// on the face (triangle) that are connected to the vertex.
///
/// @see http://www.bytehazard.com/articles/vertnorm.html
fn weight_normal_using_surface_and_angle(
    face_normal: Vector3<f32>,
    connected_face_edge_1: Vector3<f32>,
    connected_face_edge_2: Vector3<f32>,
) -> Vector3<f32> {
    let face_normal = face_normal.normalize();

    let angle = connected_face_edge_1.angle(&connected_face_edge_2);

    let area =
        0.5 * connected_face_edge_1.magnitude() * connected_face_edge_2.magnitude() * angle.sin();

    face_normal * area * angle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertex_attributes::IndexedAttribute;
    use crate::{BlenderMesh, CreateSingleIndexConfig};
    use std::f32::consts::PI;

    /// Calculate a weighted normal from a given normal and the connected edges
    #[test]
    fn calculate_weighted_normal() {
        let input_normal: Vector3<f32> = [0., 1., 0.].into();

        let connected_face_edge_1 = [1., 0., 0.].into();
        let connected_face_edge_2 = [0., 0., 1.].into();

        let expected_angle = PI / 2.;
        let expected_area = 0.5;

        let weighted_normal = weight_normal_using_surface_and_angle(
            input_normal.clone(),
            connected_face_edge_1,
            connected_face_edge_2,
        );

        assert_eq!(
            weighted_normal,
            input_normal * expected_area * expected_angle
        );
    }

    /// Given vertex indices, positions and normals - calculate the weighted normals.
    ///
    /// We provide a single triangle, so there are no overlapping vertices, so no normals are
    /// weighted and we end with the data that we started with.
    #[test]
    fn weighted_normals_one_triangle() {
        let indices = vec![0, 1, 2];

        #[rustfmt::skip]
        let positions = vec![
            0., 0., 0.,
            1., 0., 0.,
            0., 1., 0.,
        ];

        let normal_indices = vec![2, 0, 1];
        #[rustfmt::skip]
        let normals = vec![
            0., 1., 0.,
            1., 0., 0.,
            0., 1., 0.,
        ];

        let mut single_indexed = SingleIndexedVertexAttributes {
            indices,
            positions: (positions, 3).into(),
            normals: Some((normals.clone(), 3).into()),
            ..SingleIndexedVertexAttributes::default()
        };
        single_indexed.face_weight_normals().unwrap();

        assert_eq!(single_indexed.normals.unwrap().data, normals);
    }

    /// We repeat position index 0 twice - meaning that there are two vertices that share
    /// a position.
    /// The corresponding normals should be blended
    ///
    /// We have a vertex [0., 0., 0.] 0 that is part of two triangles.
    ///   The two connected vertices for the first triangle are [1., 0., 0.] and [0., 1., 0.]
    ///   The two connected vertices for the second triangle are [10., 0., 0.] and [0., 10., 0.]
    ///
    /// Since the area of the second triangle is 100x the area of the first, the normal of the
    /// second triangle should have 100x the influence.
    /// (both triangles have the same angle, so only the surface area applies here)
    #[test]
    fn weights_two_normals() {
        let mut single_indexed = create_single_indexed();
        single_indexed.face_weight_normals().unwrap();

        let expected_weighted_norm = get_expected_weighted_norm();

        let actual_normals = single_indexed.normals.unwrap().data;
        assert_eq!(&actual_normals[0..3], &expected_weighted_norm);
        assert_eq!(&actual_normals[12..15], &expected_weighted_norm);
    }

    fn get_expected_weighted_norm() -> [f32; 3] {
        let angle = PI / 2.;

        let area = 1. * 1. / 2.;
        let first_triangle_contrib = Vector3::new(0., 1., 0.) * area * angle;

        let area = 10. * 10. / 2.;
        let second_triangle_contrib = Vector3::new(1., 0., 0.) * area * angle;

        let weighted_normal = (first_triangle_contrib + second_triangle_contrib).normalize();
        let expected_weighted_norm = [weighted_normal[0], weighted_normal[1], weighted_normal[2]];

        expected_weighted_norm
    }

    fn create_single_indexed() -> SingleIndexedVertexAttributes {
        // index 0 and index 4 both correspond to position (0., 0., 0.)
        let indices = vec![0, 1, 2, 3, 4, 5];

        #[rustfmt::skip]
        let positions = vec![
            0., 0., 0.,
            1., 0., 0.,
            0., 1., 0.,
            10., 0., 0.,
            0., 0., 0.,
            0., 10., 0.,
        ];

        #[rustfmt::skip]
        let normals = vec![
            0., 1., 0.,
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            1., 0., 0.,
            0., 0., 0.,
        ];

        let single_indexed = SingleIndexedVertexAttributes {
            indices,
            positions: (positions, 3).into(),
            normals: Some((normals, 3).into()),
            ..SingleIndexedVertexAttributes::default()
        };
        single_indexed
    }
}
