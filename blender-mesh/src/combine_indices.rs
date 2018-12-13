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

        let start_size = max(self.vertex_positions.len(), self.vertex_normals.len());

        let mut expanded_normals = vec![];
        expanded_normals.resize(start_size, 12345.0);

        let mut expanded_uvs = vec![];
        expanded_uvs.resize(start_size * 2 / 3 as usize, 12345.0);

        let mut expanded_pos_indices = vec![];

        // TODO: Revisit these clones when we refactor..
        let mut new_group_indices = self.vertex_group_indices.clone();
        let mut new_group_weights = self.vertex_group_weights.clone();
        let mut new_groups_for_each_vert = self.num_groups_for_each_vertex.clone();

        expanded_pos_indices.resize(self.vertex_position_indices.len(), 0);

        let vert_group_map = self.vertex_group_as_hashmap();

        for (elem_array_index, vert_id) in self.vertex_position_indices.iter().enumerate() {
            let vert_id = *vert_id;
            let normal_index = self.vertex_normal_indices.as_ref().unwrap()[elem_array_index];
            let uv_index = match self.vertex_uv_indices.as_ref() {
                Some(uvs) => Some(uvs[elem_array_index]),
                None => None,
            };

            let vert_id_to_reuse = encountered_vert_data
                .get(&(vert_id, normal_index, uv_index))
                .cloned();

            // If we have a vertex that is already using the same indices that this current vertex is using
            // OR we have never seen this vertex index we will either:
            //  1. Re-use it
            //  OR 2. Use this newly encountered index and add it to our encountered indices / data
            let can_use_vert_id =
                vert_id_to_reuse.is_some() || !encountered_vert_ids.contains(&vert_id);

            if can_use_vert_id {
                let vert_id = match vert_id_to_reuse {
                    Some(i) => i,
                    None => vert_id,
                };

                expanded_pos_indices[elem_array_index] = vert_id;

                // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num

                expanded_normals[vert_id as usize * 3] = self.vertex_x_normal(normal_index);
                expanded_normals[vert_id as usize * 3 + 1] = self.vertex_y_normal(normal_index);
                expanded_normals[vert_id as usize * 3 + 2] = self.vertex_z_normal(normal_index);

                if has_uvs {
                    let uv_index = uv_index.unwrap();
                    expanded_uvs[vert_id as usize * 2] = self.vertex_x_uv(uv_index);
                    expanded_uvs[vert_id as usize * 2 + 1] = self.vertex_y_uv(uv_index);
                }

                encountered_vert_ids.insert(vert_id);
                encountered_vert_data.insert((vert_id, normal_index, uv_index), vert_id);

                continue;
            }

            // If we've encountered an existing position index but the normal / uv indices for this
            // vertex aren't the same as ones that we've previously encountered we'll need to
            // create a new vertex index with this new combination of data.

            largest_vert_id += 1;

            expanded_pos_indices[elem_array_index] = largest_vert_id as u16;

            let (x, y, z) = self.vertex_pos_at_idx(vert_id);
            self.vertex_positions.push(x);
            self.vertex_positions.push(y);
            self.vertex_positions.push(z);

            expanded_normals.push(self.vertex_x_normal(normal_index));
            expanded_normals.push(self.vertex_y_normal(normal_index));
            expanded_normals.push(self.vertex_z_normal(normal_index));

            if has_uvs {
                let uv_index = uv_index.unwrap();
                expanded_uvs.push(self.vertex_x_uv(uv_index));
                expanded_uvs.push(self.vertex_y_uv(uv_index));
            }

            // TODO: Move this into its own function out of our way..
            match self.num_groups_for_each_vertex.as_ref() {
                Some(num_groups_for_each_vertex) => {
                    let pos_index = vert_id as usize;
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
                (vert_id as u16, normal_index, uv_index),
                largest_vert_id as u16,
            );
        }

        self.vertex_position_indices = expanded_pos_indices;
        self.vertex_normals = expanded_normals;

        self.vertex_positions.resize(largest_vert_id * 3 + 3, 0.0);
        self.vertex_normals.resize(largest_vert_id * 3 + 3, 0.0);

        self.vertex_group_indices = new_group_indices;
        self.num_groups_for_each_vertex = new_groups_for_each_vert;
        self.vertex_group_weights = new_group_weights;

        if has_uvs {
            self.vertex_uvs = Some(expanded_uvs);
            self.vertex_uvs
                .as_mut()
                .unwrap()
                .resize(largest_vert_id * 2 + 2, 0.0);
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

    fn vertex_pos_at_idx(&self, vertex_id: u16) -> (f32, f32, f32) {
        (
            self.vertex_x_pos(vertex_id),
            self.vertex_y_pos(vertex_id),
            self.vertex_z_pos(vertex_id),
        )
    }
    fn vertex_x_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 0]
    }
    fn vertex_y_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 1]
    }
    fn vertex_z_pos(&self, vertex_id: u16) -> f32 {
        self.vertex_positions[vertex_id as usize * 3 + 2]
    }
    fn vertex_x_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 0]
    }
    fn vertex_y_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 1]
    }
    fn vertex_z_normal(&self, vertex_id: u16) -> f32 {
        self.vertex_normals[vertex_id as usize * 3 + 2]
    }
    fn vertex_x_uv(&self, vertex_id: u16) -> f32 {
        self.vertex_uvs.as_ref().unwrap()[vertex_id as usize * 2 + 0]
    }
    fn vertex_y_uv(&self, vertex_id: u16) -> f32 {
        self.vertex_uvs.as_ref().unwrap()[vertex_id as usize * 2 + 1]
    }
}
