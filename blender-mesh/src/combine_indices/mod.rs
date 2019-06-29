use self::create_single_index_config::CreateSingleIndexConfig;
use crate::bone::BoneInfluencesPerVertex;
use crate::vertex_data::{AttributeSize, VertexAttribute};
use crate::BlenderMesh;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

mod create_single_index_config;

/// Used to set temporary data that should get overwritten.
///
/// So, if we ever see this number in our data it should make it easier to see that the
/// data was improperly generated somehow.
///
/// Our unit tests should prevemt this, so this is a safety precaution.
const EASILY_RECOGNIZABLE_NUMBER: f32 = 123456789.;

impl BlenderMesh {
    /// We export our models with indices for positions, normals and uvs because
    ///
    ///  1) Easier because we we can unit test that here vs. a blender python script that's much
    ///     trickier to test.
    ///  2) Reduces amount of data required to represent the model on disk.
    ///
    /// OpenGL only supports one index buffer, we convert our vertex data
    /// from having three indices to having one. This usually requires some duplication of
    /// vertex data. We duplicate the minimum amount of vertex data necessary.
    ///
    /// FIXME: Make this function set BlenderMesh.vertex_data = VertexData::SingleIndexVertexData
    pub fn combine_vertex_indices(&mut self, config: &CreateSingleIndexConfig) {
        if let Some(bone_influences_per_vertex) = config.bone_influences_per_vertex {
            self.set_bone_influences_per_vertex(bone_influences_per_vertex);
        }

        let has_uvs = self.vertex_uvs.is_some();

        let mut largest_vert_id = *self.vertex_position_indices.iter().max().unwrap() as usize;

        let mut encountered_vert_data = EncounteredIndexCombinations::default();

        let mut encountered_vert_ids = HashSet::new();

        let mut expanded_positions = vec![];
        expanded_positions.resize((largest_vert_id + 1) * 3, EASILY_RECOGNIZABLE_NUMBER);
        let mut expanded_positions = VertexAttribute::new(expanded_positions, AttributeSize::Three);

        let mut expanded_normals = vec![];
        expanded_normals.resize((largest_vert_id + 1) * 3, EASILY_RECOGNIZABLE_NUMBER);
        let mut expanded_normals = VertexAttribute::new(expanded_normals, AttributeSize::Three);

        let mut expanded_uvs = vec![];
        expanded_uvs.resize((largest_vert_id + 1) * 2, EASILY_RECOGNIZABLE_NUMBER);
        let mut expanded_uvs = VertexAttribute::new(expanded_uvs, AttributeSize::Two);

        let mut expanded_pos_indices = vec![];

        let mut new_group_indices = self.vertex_group_indices.clone();
        let mut new_group_weights = self.vertex_group_weights.clone();

        expanded_pos_indices.resize(self.vertex_position_indices.len(), 0);

        for (elem_array_index, start_vert_id) in self.vertex_position_indices.iter().enumerate() {
            let start_vert_id = *start_vert_id;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[elem_array_index];
            let uv_index = match self.vertex_uv_indices.as_ref() {
                Some(uvs) => Some(uvs[elem_array_index]),
                None => None,
            };

            let vert_id_to_reuse =
                encountered_vert_data.get(&(start_vert_id, normal_index, uv_index));

            // If we have a vertex that is already using the same indices that this current vertex is using
            // OR we have never seen this vertex index we will either:
            //  1. Re-use it
            //  OR 2. Use this newly encountered index and add it to our encountered indices / data

            // If we've already seen this combination of vertex indices we'll re-use the index
            if vert_id_to_reuse.is_some() {
                expanded_pos_indices[elem_array_index] = *vert_id_to_reuse.unwrap();
                continue;
            }

            // If this is our first time seeing this combination of vertex indices we'll insert
            // the expanded data
            if !encountered_vert_ids.contains(&start_vert_id) {
                encountered_vert_ids.insert(start_vert_id);

                self.handle_first_vertex_encounter(
                    &mut encountered_vert_data,
                    &mut expanded_pos_indices,
                    start_vert_id,
                    elem_array_index,
                    &mut expanded_positions,
                    &mut expanded_normals,
                    &mut expanded_uvs,
                    normal_index,
                    uv_index,
                );

                continue;
            }

            // If we've encountered an existing position index but the normal / uv indices for this
            // vertex aren't the same as ones that we've previously encountered we'll need to
            // create a new vertex index with this new combination of data.

            largest_vert_id += 1;

            expanded_pos_indices[elem_array_index] = largest_vert_id as u16;

            let (x, y, z) = self.vertex_pos_at_idx(start_vert_id);
            expanded_positions.push(x);
            expanded_positions.push(y);
            expanded_positions.push(z);

            let (x, y, z) = self.vertex_normal_at_idx(normal_index);
            expanded_normals.push(x);
            expanded_normals.push(y);
            expanded_normals.push(z);

            if has_uvs {
                let uv_index = uv_index.unwrap();
                let (u, v) = self.vertex_uv_at_idx(uv_index);
                expanded_uvs.push(u);
                expanded_uvs.push(v);
            }

            // If the mesh has bone influences append bone data to the end of the bone vectors
            // to account for this newly generated vertex.
            // TODO: BREADCRUMB -  Move this into its own function out of our way..
            if let Some(bone_influences_per_vertex) = config.bone_influences_per_vertex {
                let vert_idx = start_vert_id as usize;
                // Where in our vector of group indices / weights does this vertex start?
                let group_data_start_idx = vert_idx * bone_influences_per_vertex as usize;

                for i in 0..bone_influences_per_vertex {
                    let group_data_idx = group_data_start_idx + i as usize;
                    let weight = new_group_weights.as_ref().unwrap()[group_data_idx];
                    new_group_weights.as_mut().unwrap().push(weight);

                    let index = new_group_indices.as_ref().unwrap()[group_data_idx];
                    new_group_indices.as_mut().unwrap().push(index);
                }
            }

            encountered_vert_data.insert(
                (start_vert_id as u16, normal_index, uv_index),
                largest_vert_id as u16,
            );
        }

        self.vertex_position_indices = expanded_pos_indices;

        self.vertex_normals = expanded_normals.data().clone();
        self.vertex_positions = expanded_positions.data().clone();

        self.vertex_group_indices = new_group_indices;
        self.vertex_group_weights = new_group_weights;

        if has_uvs {
            self.vertex_uvs = Some(expanded_uvs.data().clone());
        }

        self.vertex_normal_indices = None;
        self.vertex_uv_indices = None;
    }

    // TODO: Way too many parameters - just working on splitting things up into smaller functions..
    fn handle_first_vertex_encounter(
        &self,
        encountered_vert_data: &mut EncounteredIndexCombinations,
        expanded_pos_indices: &mut Vec<u16>,
        start_vert_id: u16,
        elem_array_index: usize,
        expanded_positions: &mut VertexAttribute,
        expanded_normals: &mut VertexAttribute,
        expanded_uvs: &mut VertexAttribute,
        normal_index: u16,
        uv_index: Option<u16>,
    ) {
        let has_uvs = self.vertex_uvs.is_some();

        expanded_pos_indices[elem_array_index] = start_vert_id;

        let start_vert_id = start_vert_id as usize;

        // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num
        let (x, y, z) = self.vertex_pos_at_idx(start_vert_id as u16);
        expanded_positions.set_three_components(start_vert_id, x, y, z);

        let (x, y, z) = self.vertex_normal_at_idx(normal_index);
        expanded_normals.set_three_components(start_vert_id, x, y, z);

        if has_uvs {
            let uv_index = uv_index.unwrap();
            let (u, v) = self.vertex_uv_at_idx(uv_index);
            expanded_uvs.set_two_components(start_vert_id, u, v);
        }

        let start_vert_id = start_vert_id as u16;

        encountered_vert_data.insert((start_vert_id, normal_index, uv_index), start_vert_id);
    }
}

type PosIndex = u16;
type NormalIndex = u16;
type UvIndex = Option<u16>;
#[derive(Debug, Default)]
struct EncounteredIndexCombinations {
    encountered: HashMap<(PosIndex, NormalIndex, UvIndex), PosIndex>,
}

impl Deref for EncounteredIndexCombinations {
    type Target = HashMap<(PosIndex, NormalIndex, UvIndex), PosIndex>;

    fn deref(&self) -> &Self::Target {
        &self.encountered
    }
}

impl DerefMut for EncounteredIndexCombinations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encountered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concat_vecs;
    use crate::test_utils::*;

    struct CombineIndicesTest {
        mesh_to_combine: BlenderMesh,
        expected_combined_mesh: BlenderMesh,
        create_single_idx_config: Option<CreateSingleIndexConfig>,
    }

    impl CombineIndicesTest {
        fn test(&mut self) {
            self.mesh_to_combine.combine_vertex_indices(
                self.create_single_idx_config
                    .as_ref()
                    .unwrap_or(&CreateSingleIndexConfig::default()),
            );
            let combined_mesh = &self.mesh_to_combine;
            assert_eq!(combined_mesh, &self.expected_combined_mesh);
        }
    }

    #[test]
    fn combine_pos_norm_indices() {
        let mesh_to_combine = make_mesh_to_combine_without_uvs();
        let expected_combined_mesh = make_expected_combined_mesh();

        let create_single_idx_config = Some(CreateSingleIndexConfig {
            bone_influences_per_vertex: Some(3),
        });

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
            create_single_idx_config,
        }
        .test();
    }

    // Verify that we do not panic if we're combining indices where some of the indices have
    // larger indices coming before smaller indices.
    //
    // This ensures that we properly resize our final data vectors before we start pushing data
    // to them, vs. trying to set data into an index that is larger than the length of the
    // vector at the time.
    #[test]
    fn combine_mesh_with_non_sequential_indices() {
        let mesh_to_combine = BlenderMesh {
            vertex_positions: concat_vecs!(v(5), v(6), v(7)),
            vertex_normals: concat_vecs!(v(10), v(11), v(12)),
            vertex_uvs: Some(concat_vecs!(v2(15), v2(16), v2(17))),
            num_vertices_in_each_face: vec![3],
            vertex_position_indices: vec![2, 1, 0],
            vertex_normal_indices: Some(vec![2, 1, 0]),
            vertex_uv_indices: Some(vec![2, 1, 0]),
            ..BlenderMesh::default()
        };

        let expected_combined_mesh = BlenderMesh {
            vertex_position_indices: vec![2, 1, 0],
            vertex_positions: concat_vecs!(v(5), v(6), v(7)),
            vertex_normals: concat_vecs!(v(10), v(11), v(12)),
            vertex_uvs: Some(concat_vecs!(v2(15), v2(16), v2(17))),
            num_vertices_in_each_face: vec![3],
            ..BlenderMesh::default()
        };

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,

            create_single_idx_config: None,
        }
        .test();
    }

    // We create a mesh that might have been triangulated before it was exported from Blender.
    // Before this test we weren't combining our normals properly after using the `triangulate`
    // modifier in Blender.
    #[test]
    fn combine_already_triangulated_mesh() {
        let mesh_to_combine = BlenderMesh {
            vertex_positions: concat_vecs!(v(5), v(6), v(7), v(8)),
            vertex_normals: concat_vecs!(v(10), v(11), v(12), v(13), v(14), v(15), v(16), v(17)),
            num_vertices_in_each_face: vec![3, 3, 3],
            vertex_position_indices: concat_vecs!(vec![0, 1, 2], vec![0, 2, 3], vec![0, 2, 3]),
            vertex_normal_indices: Some(concat_vecs!(vec![0, 1, 2], vec![0, 2, 3], vec![4, 5, 6])),
            ..BlenderMesh::default()
        };

        let expected_combined_mesh = BlenderMesh {
            vertex_positions: concat_vecs!(v3_x3(5, 6, 7), v(8), v3_x3(5, 7, 8)),
            vertex_position_indices: concat_vecs![vec![0, 1, 2], vec![0, 2, 3], vec![4, 5, 6]],
            num_vertices_in_each_face: vec![3, 3, 3],
            vertex_normals: concat_vecs!(v3_x3(10, 11, 12), v(13), v3_x3(14, 15, 16)),
            ..BlenderMesh::default()
        };

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
            create_single_idx_config: None,
        }
        .test();
    }

    // We create a mesh where our first three triangles have no repeating vertices
    // (across norms, uvs and positions) then our fourth triangle has all repeating vertices
    #[test]
    fn combine_pos_norm_uv_indices() {
        let mesh_to_combine = BlenderMesh {
            vertex_positions: concat_vecs!(v(0), v(1), v(2), v(3)),
            vertex_normals: concat_vecs!(v(4), v(5), v(6)),
            num_vertices_in_each_face: vec![4, 4, 4, 4],
            vertex_position_indices: concat_vecs!(
                vec![0, 1, 2, 3],
                vec![0, 1, 2, 3],
                vec![0, 1, 2, 3],
                vec![0, 1, 2, 3]
            ),
            vertex_normal_indices: Some(concat_vecs!(
                vec![0, 1, 0, 1],
                vec![2, 2, 2, 2],
                vec![2, 2, 2, 2],
                vec![2, 2, 2, 2]
            )),
            vertex_uvs: Some(concat_vecs!(v2(7), v2(8), v2(9), v2(10))),
            vertex_uv_indices: Some(concat_vecs!(
                vec![0, 1, 0, 1],
                vec![2, 2, 2, 2],
                vec![3, 3, 3, 3],
                vec![3, 3, 3, 3]
            )),
            // We already tested vertex group indices / weights about so not bothering setting up
            // more test data
            ..BlenderMesh::default()
        };

        let expected_combined_mesh = BlenderMesh {
            vertex_positions: concat_vecs!(v3_x4(0, 1, 2, 3), v3_x4(0, 1, 2, 3), v3_x4(0, 1, 2, 3)),
            vertex_position_indices: concat_vecs![
                // First Triangle
                vec![0, 1, 2, 3,],
                // Second Triangle
                vec![4, 5, 6, 7],
                // Third Triangle
                vec![8, 9, 10, 11],
                // Fourth Triangle
                vec![8, 9, 10, 11]
            ],
            num_vertices_in_each_face: vec![4, 4, 4, 4],
            vertex_normals: concat_vecs!(v3_x4(4, 5, 4, 5), v3_x4(6, 6, 6, 6), v3_x4(6, 6, 6, 6)),
            vertex_uvs: Some(concat_vecs!(
                v2_x4(7, 8, 7, 8),
                v2_x4(9, 9, 9, 9),
                v2_x4(10, 10, 10, 10)
            )),
            ..BlenderMesh::default()
        };

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
            create_single_idx_config: None,
        }
        .test();
    }

    /// Given a mesh, first calculate its face tangents.
    /// Then, when we combine indices, calculate the per vertex tangents.
    #[test]
    fn calculate_per_vertex_tangents() {
        // TODO:
        assert_eq!(2, 2);
    }

    fn make_mesh_to_combine_without_uvs() -> BlenderMesh {
        let start_positions = concat_vecs!(v(0), v(1), v(2), v(3));
        let start_normals = concat_vecs!(v(4), v(5), v(6));

        BlenderMesh {
            vertex_positions: start_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: start_normals,
            // Our last 4 vertices already exist so our expected mesh will generate
            // position indices 4, 5, 6 and 7 and use those for the second to last 4 and
            // then last 4 indices
            vertex_normal_indices: Some(vec![0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2]),
            bone_influences_per_vertex: Some(vec![3, 2, 5, 1].into()),
            vertex_group_indices: Some(vec![0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11]),
            vertex_group_weights: Some(vec![
                0.05, 0.8, 0.15, 0.5, 0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999,
            ]),
            ..BlenderMesh::default()
        }
    }

    fn make_expected_combined_mesh() -> BlenderMesh {
        let end_positions = concat_vecs!(v(0), v(1), v(2), v(3), v(0), v(1), v(2), v(3));
        let end_normals = concat_vecs!(v(4), v(5), v(4), v(5), v(6), v(6), v(6), v(6));

        BlenderMesh {
            vertex_positions: end_positions,
            vertex_position_indices: vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7],
            num_vertices_in_each_face: vec![4, 4, 4],
            vertex_normals: end_normals,
            bone_influences_per_vertex: Some(BoneInfluencesPerVertex::Uniform(3)),
            // Config.bone_influences_per_vertex = 3
            vertex_group_indices: Some(vec![
                1, 2, 0, 0, 3, 0, 8, 5, 6, 11, 0, 0, 1, 2, 0, 0, 3, 0, 8, 5, 6, 11, 0, 0,
            ]),
            // Config.bone_influences_per_vertex = 3
            vertex_group_weights: Some(vec![
                0.8, 0.15, 0.05, 0.5, 0.5, 0.0, 0.3, 0.2, 0.2, 0.999, 0.0, 0.0, 0.8, 0.15, 0.05,
                0.5, 0.5, 0.0, 0.3, 0.2, 0.2, 0.999, 0.0, 0.0,
            ]),
            ..BlenderMesh::default()
        }
    }
}
