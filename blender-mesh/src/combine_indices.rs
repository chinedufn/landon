use crate::BlenderMesh;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;

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
    /// FIXME: Wrote a test and threw code at the wall until it passed. Need to refactor
    /// this extensively! Any work on this before refactoring will not be worth the time
    /// Split this up into smaller functions that it calls, and clean up those functions.
    pub fn combine_vertex_indices(&mut self) {
        type PosIndex = u16;
        type NormalIndex = u16;
        type UvIndex = Option<u16>;
        type EncounteredIndices = HashMap<(PosIndex, NormalIndex, UvIndex), PosIndex>;

        let has_uvs = self.vertex_uvs.is_some();

        let mut largest_vert_id = *self.vertex_position_indices.iter().max().unwrap() as usize;

        let mut encountered_vert_data: EncounteredIndices = HashMap::new();
        let mut encountered_vert_ids = HashSet::new();

        let mut expanded_positions = HashMap::new();
        let mut expanded_normals = HashMap::new();
        let mut expanded_uvs = HashMap::new();

        let mut expanded_pos_indices = vec![];

        // TODO: Revisit these clones when we refactor..
        let mut new_group_indices = self.vertex_group_indices.clone();
        let mut new_group_weights = self.vertex_group_weights.clone();
        let mut new_groups_for_each_vert = self.num_groups_for_each_vertex.clone();

        expanded_pos_indices.resize(self.vertex_position_indices.len(), 0);

        let vert_group_map = self.vertex_group_as_hashmap();

        for (elem_array_index, start_vert_id) in self.vertex_position_indices.iter().enumerate() {
            let start_vert_id = *start_vert_id;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[elem_array_index];
            let uv_index = match self.vertex_uv_indices.as_ref() {
                Some(uvs) => Some(uvs[elem_array_index]),
                None => None,
            };

            let vert_id_to_reuse = encountered_vert_data
                .get(&(start_vert_id, normal_index, uv_index))
                .cloned();

            // If we have a vertex that is already using the same indices that this current vertex is using
            // OR we have never seen this vertex index we will either:
            //  1. Re-use it
            //  OR 2. Use this newly encountered index and add it to our encountered indices / data

            // If we've already seen this combination of vertex indices we'll re-use the index
            if vert_id_to_reuse.is_some() {
                expanded_pos_indices[elem_array_index] = vert_id_to_reuse.unwrap();
                continue;
            }

            // If this is our first time seeing this combination of vertex indices we'll insert
            // the expanded data
            if !encountered_vert_ids.contains(&start_vert_id) {
                expanded_pos_indices[elem_array_index] = start_vert_id;

                let start_vert_id = start_vert_id as usize;

                // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num
                let (x, y, z) = self.vertex_pos_at_idx(start_vert_id as u16);
                expanded_positions.insert(start_vert_id * 3, x);
                expanded_positions.insert(start_vert_id * 3 + 1, y);
                expanded_positions.insert(start_vert_id * 3 + 2, z);

                let (x, y, z) = self.vertex_normal_at_idx(normal_index);
                expanded_normals.insert(start_vert_id * 3, x);
                expanded_normals.insert(start_vert_id * 3 + 1, y);
                expanded_normals.insert(start_vert_id * 3 + 2, z);

                if has_uvs {
                    let uv_index = uv_index.unwrap();
                    let (x, y) = self.vertex_uv_at_idx(uv_index);
                    expanded_uvs.insert(start_vert_id * 2, x);
                    expanded_uvs.insert(start_vert_id * 2 + 1, y);
                }

                let start_vert_id = start_vert_id as u16;

                encountered_vert_ids.insert(start_vert_id);
                encountered_vert_data
                    .insert((start_vert_id, normal_index, uv_index), start_vert_id);

                continue;
            }

            // If we've encountered an existing position index but the normal / uv indices for this
            // vertex aren't the same as ones that we've previously encountered we'll need to
            // create a new vertex index with this new combination of data.

            largest_vert_id += 1;

            expanded_pos_indices[elem_array_index] = largest_vert_id as u16;

            let (x, y, z) = self.vertex_pos_at_idx(start_vert_id);
            expanded_positions.insert(largest_vert_id * 3, x);
            expanded_positions.insert(largest_vert_id * 3 + 1, y);
            expanded_positions.insert(largest_vert_id * 3 + 2, z);

            let (x, y, z) = self.vertex_normal_at_idx(normal_index);
            expanded_normals.insert(largest_vert_id * 3, x);
            expanded_normals.insert(largest_vert_id * 3 + 1, y);
            expanded_normals.insert(largest_vert_id * 3 + 2, z);

            if has_uvs {
                let uv_index = uv_index.unwrap();
                let (x, y) = self.vertex_uv_at_idx(uv_index);
                expanded_uvs.insert(largest_vert_id * 2, x);
                expanded_uvs.insert(largest_vert_id * 2 + 1, y);
            }

            // TODO: Move this into its own function out of our way..
            match self.num_groups_for_each_vertex.as_ref() {
                Some(num_groups_for_each_vertex) => {
                    let pos_index = start_vert_id as usize;
                    // Where in our vector of group indices / weights does this vertex start?
                    let group_data_start_idx =
                        *vert_group_map.as_ref().unwrap().get(&pos_index).unwrap() as usize;

                    // How many groups does this vertex have?
                    let num_groups_for_this_vertex = num_groups_for_each_vertex[pos_index as usize];
                    new_groups_for_each_vert
                        .as_mut()
                        .unwrap()
                        .push(num_groups_for_this_vertex);

                    for i in 0..num_groups_for_this_vertex {
                        let group_data_idx = group_data_start_idx + i as usize;
                        let weight = new_group_weights.as_ref().unwrap()[group_data_idx];
                        new_group_weights.as_mut().unwrap().push(weight);

                        let index = new_group_indices.as_ref().unwrap()[group_data_idx];
                        new_group_indices.as_mut().unwrap().push(index);
                    }
                }
                None => {}
            };

            encountered_vert_data.insert(
                (start_vert_id as u16, normal_index, uv_index),
                largest_vert_id as u16,
            );
        }

        self.vertex_position_indices = expanded_pos_indices;

        // We use `12345.0` so that if something goes wrong it's easier to notice that the values
        // are incorrect.
        // This helps when debugging issues in our export / pre-process pipeline.
        self.vertex_normals.resize(largest_vert_id * 3 + 3, 12345.0);
        self.vertex_positions
            .resize(largest_vert_id * 3 + 3, 12345.0);

        expanded_normals.iter().for_each(|(idx, value)| {
            self.vertex_normals[*idx] = *value;
        });
        expanded_positions.iter().for_each(|(idx, value)| {
            self.vertex_positions[*idx] = *value;
        });

        self.vertex_group_indices = new_group_indices;
        self.num_groups_for_each_vertex = new_groups_for_each_vert;
        self.vertex_group_weights = new_group_weights;

        if has_uvs {
            let mut uvs = vec![];
            uvs.resize(largest_vert_id * 2 + 2, 12345.0);

            expanded_uvs.iter().for_each(|(idx, value)| {
                uvs[*idx] = *value;
            });

            self.vertex_uvs = Some(uvs);
        }

        self.vertex_normal_indices = None;
        self.vertex_uv_indices = None;
    }

    // Create a hashmap that allows us, given some vertex index, to look up the first group index
    // and weight for that vertex.
    // This is necessary because different vertices can have different numbers of groups so we
    // need to know where in our vector of group indices/weights a particular vertex's data starts.
    fn vertex_group_as_hashmap(&self) -> Option<HashMap<usize, u32>> {
        let mut total_previous: u32 = 0;

        match self.num_groups_for_each_vertex.as_ref() {
            Some(num_groups_per) => {
                let mut map = HashMap::new();

                for (index, num) in num_groups_per.iter().enumerate() {
                    map.insert(index, total_previous);
                    total_previous += *num as u32;
                }

                Some(map)
            }
            None => None,
        }
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
    }

    fn test_combine_indices(mut combine_indices_test: CombineIndicesTest) {
        combine_indices_test
            .mesh_to_combine
            .combine_vertex_indices();
        let combined_mesh = combine_indices_test.mesh_to_combine;
        assert_eq!(combined_mesh, combine_indices_test.expected_combined_mesh);
    }

    #[test]
    fn combine_pos_norm_indices() {
        let mesh_to_combine = make_mesh_to_combine_without_uvs();
        let expected_combined_mesh = make_expected_combined_mesh();

        test_combine_indices(CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
        });
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

        test_combine_indices(CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
        });
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

        test_combine_indices(CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
        });
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
            num_groups_for_each_vertex: Some(vec![3, 2, 5, 1]),
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
            num_groups_for_each_vertex: Some(vec![3, 2, 5, 1, 3, 2, 5, 1]),
            vertex_group_indices: Some(vec![
                0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11, 0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11,
            ]),
            vertex_group_weights: Some(vec![
                0.05, 0.8, 0.15, 0.5, 0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999, 0.05, 0.8, 0.15, 0.5,
                0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999,
            ]),
            ..BlenderMesh::default()
        }
    }
}
