pub use self::create_single_index_config::CreateSingleIndexConfig;
use crate::face_tangents::face_tangent_at_idx;
use crate::vertex_attributes::{BoneAttributes, SingleIndexedVertexAttributes, VertexAttribute};
use crate::{BlenderMesh, BoneInfluence, Vertex};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

mod create_single_index_config;
mod weighted_normals;

/// Used to set temporary data that should get overwritten.
///
/// So, if we ever see this number in our data it should make it easier to see that the
/// data was improperly generated somehow.
///
/// Our unit tests should prevent this from ever happening - so this is just a safety precaution
/// to more easily notice any errors.
const EASILY_RECOGNIZABLE_NUMBER: f32 = 123456789.;

impl BlenderMesh {
    /// We store our exported Blender mesh with indices for positions, normals and uvs because
    ///
    ///  1) Easier because we we can unit test that here vs. a blender python script that's much
    ///     trickier to test.
    ///
    ///  2) Smaller to store the fields indexed than to expand them.
    ///
    /// ---
    ///
    /// Most rendering pipelines only supports one index buffer, so here we convert our vertex data
    /// from having three indices to having one.
    ///
    /// This typically requires some duplication of vertex data - we duplicate the minimum amount
    /// of vertex data necessary.
    ///
    /// TODO: Need to continue refactoring
    ///
    /// TODO: Make this function set BlenderMesh.vertex_attributes = VertexData::SingleIndexVertexData
    ///
    /// TODO: Don't work on additionally functionality until we've broken up these tests
    /// and implementation into smaller, specific pieces.
    ///
    /// TODO: There are unexpected (based on the method's name) mutations in here such as
    /// triangulation. Lot's to refactor in this crate.
    pub fn combine_vertex_indices(
        &mut self,
        config: &CreateSingleIndexConfig,
    ) -> SingleIndexedVertexAttributes {
        let mut face_tangents = None;

        if let Some(bone_influences_per_vertex) = config.bone_influences_per_vertex {
            self.multi_indexed_vertex_attributes
                .set_bone_influences_per_vertex(bone_influences_per_vertex);
        }

        // Important to calculate face tangents before we modify / weight the normals
        if config.calculate_face_tangents {
            face_tangents = Some(self.calculate_face_tangents().unwrap());
        }

        let multi = &self.multi_indexed_vertex_attributes;

        let mut largest_vert_id = *multi.positions.indices.iter().max().unwrap() as usize;

        let mut encountered_vert_data = EncounteredIndexCombinations::default();

        let mut encountered_vert_ids = HashSet::new();

        let mut expanded_positions = vec![];
        expanded_positions.resize((largest_vert_id + 1) * 3, EASILY_RECOGNIZABLE_NUMBER);

        let mut expanded_normals = vec![];
        expanded_normals.resize((largest_vert_id + 1) * 3, EASILY_RECOGNIZABLE_NUMBER);

        let mut expanded_uvs = vec![];
        expanded_uvs.resize((largest_vert_id + 1) * 2, EASILY_RECOGNIZABLE_NUMBER);

        let mut expanded_pos_indices = vec![];

        let mut new_group_indices = multi
            .bone_influences
            .as_ref()
            .map(|b| b.bone_indices.clone());
        let mut new_group_weights = multi
            .bone_influences
            .as_ref()
            .map(|b| b.bone_weights.clone());

        expanded_pos_indices.resize(multi.positions.indices.len(), 0);

        let mut face_idx = 0;
        let mut vertices_until_next_face = multi.vertices_in_each_face[0];

        let mut expanded_tangents = vec![];
        expanded_tangents.resize((largest_vert_id + 1) * 3, EASILY_RECOGNIZABLE_NUMBER);

        // FIXME: Split this loop into a function
        for (elem_array_index, start_vert_id) in multi.positions.indices.iter().enumerate() {
            let start_vert_id = *start_vert_id;
            let normal_index = match multi.normals.as_ref() {
                None => None,
                Some(normals) => Some(normals.indices[elem_array_index]),
            };

            let uv_index = match multi.uvs.as_ref() {
                Some(uvs) => Some(uvs.indices[elem_array_index]),
                None => None,
            };

            let vert_id_to_reuse =
                encountered_vert_data.get(&(start_vert_id, normal_index, uv_index));

            // If we've already seen this combination of vertex indices we'll re-use the index
            if vert_id_to_reuse.is_some() {
                expanded_pos_indices[elem_array_index] = *vert_id_to_reuse.unwrap();

                if let Some(face_tangents) = &face_tangents {
                    if face_tangents.len() > 0 {
                        let (x, y, z) = face_tangent_at_idx(face_tangents, face_idx);
                        // TODO: Should we weight these based on the surface area of the face /
                        // the angle of the vertex and it's two edges on the face? Do some research
                        // on what other people do.
                        let vert_id_to_reuse = *vert_id_to_reuse.unwrap() as usize;
                        expanded_tangents[vert_id_to_reuse * 3] += x;
                        expanded_tangents[vert_id_to_reuse * 3 + 1] += y;
                        expanded_tangents[vert_id_to_reuse * 3 + 2] += z;
                    }
                }
            } else if !encountered_vert_ids.contains(&start_vert_id) {
                // If this is our first time seeing this vertex index of vertex indices we'll insert
                // the expanded data

                encountered_vert_ids.insert(start_vert_id);

                // TODO: Use a data structure that holds some of this stuff so we don't need
                // to pass it around everywhere ..
                self.handle_first_vertex_encounter(
                    &face_tangents,
                    &mut encountered_vert_data,
                    &mut expanded_pos_indices,
                    start_vert_id,
                    elem_array_index,
                    &mut expanded_positions,
                    &mut expanded_normals,
                    &mut expanded_uvs,
                    &mut expanded_tangents,
                    normal_index,
                    uv_index,
                    face_idx,
                );
            } else {
                // If we've encountered an existing position index but the normal / uv indices for this
                // vertex aren't the same as ones that we've previously encountered we'll need to
                // create a new vertex index with this new combination of data.

                largest_vert_id += 1;

                expanded_pos_indices[elem_array_index] = largest_vert_id as u16;

                self.push_generated_vertex_data(
                    start_vert_id,
                    normal_index,
                    &face_tangents,
                    uv_index,
                    config.bone_influences_per_vertex,
                    new_group_indices.as_mut(),
                    new_group_weights.as_mut(),
                    &mut expanded_positions,
                    &mut expanded_normals,
                    &mut expanded_uvs,
                    &mut expanded_tangents,
                    face_idx,
                );

                encountered_vert_data.insert(
                    (start_vert_id as u16, normal_index, uv_index),
                    largest_vert_id as u16,
                );
            }

            if face_idx + 1 < multi.vertices_in_each_face.len() {
                vertices_until_next_face -= 1;
            }

            if vertices_until_next_face == 0 {
                face_idx += 1;
                if face_idx < multi.vertices_in_each_face.len() {
                    vertices_until_next_face = multi.vertices_in_each_face[face_idx];
                }
            }
        }

        let normals = match self.multi_indexed_vertex_attributes.normals.is_some() {
            false => None,
            true => Some(expanded_normals),
        };
        let uvs = match self.multi_indexed_vertex_attributes.uvs.is_some() {
            false => None,
            true => Some(expanded_uvs),
        };

        let bones = match (
            &self.multi_indexed_vertex_attributes.bone_influences,
            config.bone_influences_per_vertex,
        ) {
            (Some(_bone_attributes), Some(bone_influences_per_vertex)) => Some((
                BoneAttributes {
                    bone_influencers: VertexAttribute::new(
                        new_group_indices.unwrap(),
                        bone_influences_per_vertex,
                    )
                    .unwrap(),
                    bone_weights: VertexAttribute::new(
                        new_group_weights.unwrap(),
                        bone_influences_per_vertex,
                    )
                    .unwrap(),
                },
                bone_influences_per_vertex,
            )),
            _ => None,
        };

        let tangents = face_tangents.map(|_| expanded_tangents);

        let mut single_indexed_vertex_attributes = SingleIndexedVertexAttributes {
            indices: expanded_pos_indices,
            vertices: make_vertices(expanded_positions, normals, uvs, tangents, bones),
        };

        let indices = self.triangulate(&single_indexed_vertex_attributes.indices);
        single_indexed_vertex_attributes.indices = indices;

        single_indexed_vertex_attributes
    }

    // TODO: Way too many parameters - just working on splitting things up into smaller functions..
    fn handle_first_vertex_encounter(
        &self,
        face_tangents: &Option<Vec<f32>>,
        encountered_vert_data: &mut EncounteredIndexCombinations,
        expanded_pos_indices: &mut Vec<u16>,
        start_vert_id: u16,
        elem_array_index: usize,
        expanded_positions: &mut Vec<f32>,
        expanded_normals: &mut Vec<f32>,
        expanded_uvs: &mut Vec<f32>,
        expanded_tangents: &mut Vec<f32>,
        normal_index: Option<u16>,
        uv_index: Option<u16>,
        face_idx: usize,
    ) {
        let multi = &self.multi_indexed_vertex_attributes;

        expanded_pos_indices[elem_array_index] = start_vert_id;

        let start_vert_id = start_vert_id as usize;

        // TODO: Six methods to get and set the normal, pos, and uv for a vertex_num
        if let &[x, y, z] = multi.positions.attribute.data_at_idx(start_vert_id as u16) {
            expanded_positions[start_vert_id * 3] = x;
            expanded_positions[start_vert_id * 3 + 1] = y;
            expanded_positions[start_vert_id * 3 + 2] = z;
        }

        if let Some(normal_index) = normal_index {
            if let &[x, y, z] = multi
                .normals
                .as_ref()
                .unwrap()
                .attribute
                .data_at_idx(normal_index)
            {
                expanded_normals[start_vert_id * 3] = x;
                expanded_normals[start_vert_id * 3 + 1] = y;
                expanded_normals[start_vert_id * 3 + 2] = z;
            }
        }

        if let Some(uv_index) = uv_index {
            if let &[u, v] = multi.uvs.as_ref().unwrap().attribute.data_at_idx(uv_index) {
                expanded_uvs[start_vert_id * 2] = u;
                expanded_uvs[start_vert_id * 2 + 1] = v;
            }
        }

        if let Some(face_tangents) = face_tangents {
            if face_tangents.len() > 0 {
                let (x, y, z) = face_tangent_at_idx(&face_tangents, face_idx);
                expanded_tangents[start_vert_id * 3] = x;
                expanded_tangents[start_vert_id * 3 + 1] = y;
                expanded_tangents[start_vert_id * 3 + 2] = z;
            }
        }

        let start_vert_id = start_vert_id as u16;

        encountered_vert_data.insert((start_vert_id, normal_index, uv_index), start_vert_id);
    }

    // TODO: Way too many parameters - just working on splitting things up into smaller functions..
    fn push_generated_vertex_data(
        &self,
        pos_idx: u16,
        normal_idx: Option<u16>,
        face_tangents: &Option<Vec<f32>>,
        uv_idx: Option<u16>,
        bone_influences_per_vertex: Option<u8>,
        new_group_indices: Option<&mut Vec<u8>>,
        new_group_weights: Option<&mut Vec<f32>>,
        expanded_positions: &mut Vec<f32>,
        expanded_normals: &mut Vec<f32>,
        expanded_uvs: &mut Vec<f32>,
        expanded_tangents: &mut Vec<f32>,
        face_idx: usize,
    ) {
        let multi = &self.multi_indexed_vertex_attributes;

        if let &[x, y, z] = multi.positions.attribute.data_at_idx(pos_idx) {
            expanded_positions.push(x);
            expanded_positions.push(y);
            expanded_positions.push(z);
        }

        if let Some(normal_idx) = normal_idx {
            if let &[x, y, z] = multi
                .normals
                .as_ref()
                .unwrap()
                .attribute
                .data_at_idx(normal_idx)
            {
                expanded_normals.push(x);
                expanded_normals.push(y);
                expanded_normals.push(z);
            }
        }

        if let Some(uvs) = &multi.uvs {
            let uv_index = uv_idx.unwrap();
            if let &[u, v] = uvs.attribute.data_at_idx(uv_index) {
                expanded_uvs.push(u);
                expanded_uvs.push(v);
            }
        }

        if let Some(face_tangents) = face_tangents {
            if face_tangents.len() > 0 {
                let (x, y, z) = face_tangent_at_idx(face_tangents, face_idx);
                expanded_tangents.push(x);
                expanded_tangents.push(y);
                expanded_tangents.push(z);
            }
        }

        // If the mesh has bone influences append bone data to the end of the bone vectors
        // to account for this newly generated vertex.
        if let Some(bone_influences_per_vertex) = bone_influences_per_vertex {
            self.push_bone_data_for_generated_vertex(
                pos_idx as usize,
                bone_influences_per_vertex,
                new_group_indices.unwrap(),
                new_group_weights.unwrap(),
            );
        }
    }

    // TODO: Way too many parameters - just working on splitting things up into smaller functions..
    fn push_bone_data_for_generated_vertex(
        &self,
        vert_idx: usize,
        bone_influences_per_vertex: u8,
        new_group_indices: &mut Vec<u8>,
        new_group_weights: &mut Vec<f32>,
    ) {
        // Where in our vector of group indices / weights does this vertex start?
        let group_data_start_idx = vert_idx * bone_influences_per_vertex as usize;

        for i in 0..bone_influences_per_vertex {
            let group_data_idx = group_data_start_idx + i as usize;
            let weight = new_group_weights[group_data_idx];
            new_group_weights.push(weight);

            let index = new_group_indices[group_data_idx];
            new_group_indices.push(index);
        }
    }
}

type PosIndex = u16;
type NormalIndex = Option<u16>;
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

// TODO: We're just throwing things around as we work to refactor this crate ...
fn make_vertices(
    vertex_positions: Vec<f32>,
    vertex_normals: Option<Vec<f32>>,
    vertex_uvs: Option<Vec<f32>>,
    tangents: Option<Vec<f32>>,
    bones: Option<(BoneAttributes, u8)>,
) -> Vec<Vertex> {
    let mut vertices = vec![];
    for idx in 0..vertex_positions.len() / 3 {
        let position = [
            vertex_positions[idx * 3],
            vertex_positions[idx * 3 + 1],
            vertex_positions[idx * 3 + 2],
        ];
        let normal = vertex_normals
            .as_ref()
            .map(|n| [n[idx * 3], n[idx * 3 + 1], n[idx * 3 + 2]]);
        let uv = vertex_uvs
            .as_ref()
            .map(|uvs| [uvs[idx * 2], uvs[idx * 2 + 1]]);
        let face_tangent = tangents.as_ref().map(|face_tangents| {
            [
                face_tangents[idx * 3],
                face_tangents[idx * 3 + 1],
                face_tangents[idx * 3 + 2],
            ]
        });
        let bones = bones.as_ref().map(|(b, influences_per_vertex)| {
            let count = *influences_per_vertex;

            let mut bones = [BoneInfluence {
                bone_idx: 0,
                weight: 0.0,
            }; 4];
            for bone_idx in 0..count as usize {
                bones[bone_idx] = BoneInfluence {
                    bone_idx: b.bone_influencers[idx * count as usize + bone_idx],
                    weight: b.bone_weights[idx * count as usize + bone_idx],
                };
            }

            bones
        });
        vertices.push(Vertex {
            position,
            normal,
            face_tangent,
            uv,
            bones,
        });
    }

    vertices
}

/// TODO: These tests are getting hard to manage.
/// We need smaller tests that test individual pieces of the combining.
/// Then we can keep it to only a handful of tests that test entire meshes.
/// TODO: Don't work on additionally functionality until we've broken up these tests
/// and implementation into smaller, specific pieces.
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::bone::BoneInfluencesPerVertex;
    use crate::concat_vecs;
    use crate::test_utils::*;
    use crate::vertex_attributes::{
        BoneAttributes, IndexedAttribute, MultiIndexedVertexAttributes, VertexBoneInfluences,
    };

    struct CombineIndicesTest {
        mesh_to_combine: BlenderMesh,
        expected_combined_mesh: SingleIndexedVertexAttributes,
        create_single_idx_config: Option<CreateSingleIndexConfig>,
    }

    impl CombineIndicesTest {
        fn test(mut self) {
            let combined = self.mesh_to_combine.combine_vertex_indices(
                self.create_single_idx_config
                    .as_ref()
                    .unwrap_or(&CreateSingleIndexConfig::default()),
            );

            assert_eq!(combined, self.expected_combined_mesh);
        }
    }

    #[test]
    fn combine_pos_norm_indices() {
        let mesh_to_combine = make_mesh_to_combine_without_uvs();
        let expected_combined_mesh = make_expected_combined_mesh();

        let create_single_idx_config = Some(CreateSingleIndexConfig {
            bone_influences_per_vertex: Some(3),
            calculate_face_tangents: false,
            ..CreateSingleIndexConfig::default()
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
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_positions: concat_vecs!(v(5), v(6), v(7)),
                vertex_normals: concat_vecs!(v(10), v(11), v(12)),
                vertex_uvs: Some(concat_vecs!(v2(15), v2(16), v2(17))),
                bone_influences_per_vertex: None,
                vertex_group_indices: None,
                num_vertices_in_each_face: vec![3],
                vertex_position_indices: vec![2, 1, 0],
                vertex_normal_indices: vec![2, 1, 0],
                vertex_uv_indices: Some(vec![2, 1, 0]),
                vertex_group_weights: None,
            }
            .into(),
            ..BlenderMesh::default()
        };

        let expected_combined_mesh = TodoDeleteMeSingleConverter {
            vertex_position_indices: vec![2, 1, 0],
            vertex_positions: concat_vecs!(v(5), v(6), v(7)),
            vertex_normals: concat_vecs!(v(10), v(11), v(12)),
            vertex_uvs: Some(concat_vecs!(v2(15), v2(16), v2(17))),
            num_vertices_in_each_face: vec![3],
            tangents: None,
            bone_influences_per_vertex: None,
            vertex_group_indices: None,
            vertex_group_weights: None,
        }
        .into();

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
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_positions: concat_vecs!(v(5), v(6), v(7), v(8)),
                vertex_normals: concat_vecs!(
                    v(10),
                    v(11),
                    v(12),
                    v(13),
                    v(14),
                    v(15),
                    v(16),
                    v(17)
                ),
                num_vertices_in_each_face: vec![3, 3, 3],
                vertex_position_indices: concat_vecs!(vec![0, 1, 2], vec![0, 2, 3], vec![0, 2, 3]),
                vertex_normal_indices: concat_vecs!(vec![0, 1, 2], vec![0, 2, 3], vec![4, 5, 6]),
                ..TodoDeleteMeMultiConverter::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        let expected_combined_mesh = TodoDeleteMeSingleConverter {
            vertex_positions: concat_vecs!(v3_x3(5, 6, 7), v(8), v3_x3(5, 7, 8)),
            vertex_position_indices: concat_vecs![vec![0, 1, 2], vec![0, 2, 3], vec![4, 5, 6]],
            num_vertices_in_each_face: vec![3, 3, 3],
            vertex_normals: concat_vecs!(v3_x3(10, 11, 12), v(13), v3_x3(14, 15, 16)),
            ..TodoDeleteMeSingleConverter::default()
        }
        .into();

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
        let mesh_to_combine = mesh_to_combine_pos_norm_uv_indices();
        let expected_combined_mesh = expected_mesh_to_combine_pos_norm_uv_indices();

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
            create_single_idx_config: None,
        }
        .test();
    }

    fn mesh_to_combine_pos_norm_uv_indices() -> BlenderMesh {
        BlenderMesh {
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_positions: concat_vecs!(v(0), v(1), v(2), v(3)),
                vertex_normals: concat_vecs!(v(4), v(5), v(6)),
                num_vertices_in_each_face: vec![4, 4, 4, 4],
                vertex_position_indices: concat_vecs!(
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3]
                ),
                vertex_normal_indices: concat_vecs!(
                    vec![0, 1, 0, 1],
                    vec![2, 2, 2, 2],
                    vec![2, 2, 2, 2],
                    vec![2, 2, 2, 2]
                ),
                vertex_uvs: Some(concat_vecs!(v2(7), v2(8), v2(9), v2(10))),
                vertex_uv_indices: Some(concat_vecs!(
                    vec![0, 1, 0, 1],
                    vec![2, 2, 2, 2],
                    vec![3, 3, 3, 3],
                    vec![3, 3, 3, 3]
                )),
                ..TodoDeleteMeMultiConverter::default()
            }
            .into(),

            // We already tested vertex group indices / weights about so not bothering setting up
            // more test data
            ..BlenderMesh::default()
        }
    }

    fn expected_mesh_to_combine_pos_norm_uv_indices() -> SingleIndexedVertexAttributes {
        TodoDeleteMeSingleConverter {
            vertex_positions: concat_vecs!(v3_x4(0, 1, 2, 3), v3_x4(0, 1, 2, 3), v3_x4(0, 1, 2, 3)),
            vertex_position_indices: concat_vecs![
                // First Triangle
                vec![0, 1, 2, 0, 2, 3,],
                // Second Triangle
                vec![4, 5, 6, 4, 6, 7],
                // Third Triangle
                vec![8, 9, 10, 8, 10, 11],
                // Fourth Triangle
                vec![8, 9, 10, 8, 10, 11]
            ],
            num_vertices_in_each_face: vec![4, 4, 4, 4],
            vertex_normals: concat_vecs!(v3_x4(4, 5, 4, 5), v3_x4(6, 6, 6, 6), v3_x4(6, 6, 6, 6)),
            vertex_uvs: Some(concat_vecs!(
                v2_x4(7, 8, 7, 8),
                v2_x4(9, 9, 9, 9),
                v2_x4(10, 10, 10, 10)
            )),
            ..TodoDeleteMeSingleConverter::default()
        }
        .into()
    }

    /// Verify that when we re-use a vertex we add in the tangent of the second vertex that we're
    /// skipping to the first one that we're re-using.
    ///
    /// NOTE: Numbers in these tests were not verified by hand.
    /// Instead, we took this common tangent calculation formula wrote tests, and verified
    /// that the rendered models looked visually correct (meaning that our test values are also correct).
    #[test]
    fn calculate_per_vertex_tangents_encountered_duplicate_data() {
        let mesh_to_combine = BlenderMesh {
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_positions: concat_vecs!(
                    v(0),
                    vec![1.0, 0.0, 0.0],
                    vec![1.0, 1.0, 0.0],
                    vec![0., 1., 0.]
                ),
                vertex_normals: concat_vecs!(v(4), v(5), v(6), v(7)),
                num_vertices_in_each_face: vec![4, 4, 4, 4],
                vertex_position_indices: concat_vecs!(
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3]
                ),
                vertex_normal_indices: concat_vecs!(
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3],
                    vec![0, 1, 2, 3]
                ),
                vertex_uvs: Some(concat_vecs!(v2(0), vec![0.5, 0.0], v2(1), vec![0., 1.])),
                vertex_uv_indices: Some(concat_vecs!(
                    vec![0, 1, 2, 3], // .
                    vec![0, 1, 2, 3], // .
                    vec![0, 1, 2, 3], // .
                    vec![0, 1, 2, 3]  // .
                )),
                ..TodoDeleteMeMultiConverter::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        assert_eq!(
            mesh_to_combine.calculate_face_tangents().unwrap(),
            vec![2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0,]
        );

        let expected_combined_mesh = TodoDeleteMeSingleConverter {
            vertex_positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            vertex_position_indices: vec![
                0, 1, 2, 0, 2, 3, 0, 1, 2, 0, 2, 3, 0, 1, 2, 0, 2, 3, 0, 1, 2, 0, 2, 3,
            ],
            num_vertices_in_each_face: vec![4, 4, 4, 4],
            vertex_normals: vec![4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 6.0, 6.0, 6.0, 7.0, 7.0, 7.0],
            vertex_uvs: Some(vec![0.0, 0.0, 0.5, 0.0, 1.0, 1.0, 0.0, 1.0]),
            // 4 duplicate vertices, each with [2., 0., 0.] as the tangent
            // When combined we get [8., 0., 0.]
            tangents: Some(vec![
                8.0, 0.0, 0.0, 8.0, 0.0, 0.0, 8.0, 0.0, 0.0, 8.0, 0.0, 0.0,
            ]),
            ..TodoDeleteMeSingleConverter::default()
        }
        .into();

        let create_single_idx_config = Some(CreateSingleIndexConfig {
            bone_influences_per_vertex: None,
            calculate_face_tangents: true,
            ..CreateSingleIndexConfig::default()
        });

        CombineIndicesTest {
            mesh_to_combine,
            expected_combined_mesh,
            create_single_idx_config,
        }
        .test();
    }

    fn make_mesh_to_combine_without_uvs() -> BlenderMesh {
        let start_positions = concat_vecs!(v(0), v(1), v(2), v(3));
        let start_normals = concat_vecs!(v(4), v(5), v(6));

        BlenderMesh {
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_positions: start_positions,
                vertex_position_indices: vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
                num_vertices_in_each_face: vec![4, 4, 4],
                vertex_normals: start_normals,
                // Our last 4 vertices already exist so our expected mesh will generate
                // position indices 4, 5, 6 and 7 and use those for the second to last 4 and
                // then last 4 indices
                vertex_normal_indices: vec![0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2],
                vertex_uv_indices: None,
                vertex_uvs: None,
                bone_influences_per_vertex: Some(vec![3, 2, 5, 1].into()),
                vertex_group_indices: Some(vec![0, 1, 2, 0, 3, 4, 5, 6, 7, 8, 11]),
                vertex_group_weights: Some(vec![
                    0.05, 0.8, 0.15, 0.5, 0.5, 0.1, 0.2, 0.2, 0.2, 0.3, 0.999,
                ]),
            }
            .into(),
            ..BlenderMesh::default()
        }
    }

    fn make_expected_combined_mesh() -> SingleIndexedVertexAttributes {
        let end_positions = concat_vecs!(v(0), v(1), v(2), v(3), v(0), v(1), v(2), v(3));
        let end_normals = concat_vecs!(v(4), v(5), v(4), v(5), v(6), v(6), v(6), v(6));

        TodoDeleteMeSingleConverter {
            vertex_positions: end_positions,
            vertex_position_indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 4, 5, 6, 4, 6, 7],
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
            tangents: None,
            vertex_uvs: None,
        }
        .into()
    }

    // We changed the format of the BlenderMesh in one refactoring PR - so this holds some test data
    // with the old names and format so that we can quickly convert it into the new format.
    // TODO: Remove this and just use the new format directly
    #[derive(Default)]
    pub struct TodoDeleteMeMultiConverter {
        pub vertex_positions: Vec<f32>,
        pub vertex_position_indices: Vec<u16>,
        pub num_vertices_in_each_face: Vec<u8>,
        pub vertex_normals: Vec<f32>,
        pub vertex_normal_indices: Vec<u16>,
        pub vertex_uv_indices: Option<Vec<u16>>,
        pub vertex_uvs: Option<Vec<f32>>,
        pub(crate) bone_influences_per_vertex: Option<BoneInfluencesPerVertex>,
        // Config.bone_influences_per_vertex = 3
        pub vertex_group_indices: Option<Vec<u8>>,
        // Config.bone_influences_per_vertex = 3
        pub vertex_group_weights: Option<Vec<f32>>,
    }

    impl Into<MultiIndexedVertexAttributes> for TodoDeleteMeMultiConverter {
        fn into(self) -> MultiIndexedVertexAttributes {
            let normals = Some(IndexedAttribute {
                indices: self.vertex_normal_indices,
                attribute: (self.vertex_normals, 3).into(),
            });

            let mut uvs = None;
            let mut parent_armature_bone_influences = None;

            if self.vertex_uv_indices.is_some() {
                uvs = Some(IndexedAttribute {
                    indices: self.vertex_uv_indices.unwrap(),
                    attribute: (self.vertex_uvs.unwrap(), 2).into(),
                })
            }

            if let Some(bone_influences_per_vertex) = self.bone_influences_per_vertex {
                parent_armature_bone_influences = Some(VertexBoneInfluences {
                    bones_per_vertex: bone_influences_per_vertex,
                    bone_indices: self.vertex_group_indices.unwrap(),
                    bone_weights: self.vertex_group_weights.unwrap(),
                })
            }

            MultiIndexedVertexAttributes {
                vertices_in_each_face: self.num_vertices_in_each_face,
                positions: IndexedAttribute {
                    indices: self.vertex_position_indices,
                    attribute: (self.vertex_positions, 3).into(),
                },
                normals,
                uvs,
                bone_influences: parent_armature_bone_influences,
            }
        }
    }

    /// Messy code as part of an effort to slowly refactor these crates...
    #[derive(Default)]
    pub struct TodoDeleteMeSingleConverter {
        pub vertex_positions: Vec<f32>,
        pub vertex_position_indices: Vec<u16>,
        pub num_vertices_in_each_face: Vec<u8>,
        pub vertex_normals: Vec<f32>,
        pub vertex_uvs: Option<Vec<f32>>,
        pub tangents: Option<Vec<f32>>,
        pub(crate) bone_influences_per_vertex: Option<BoneInfluencesPerVertex>,
        pub vertex_group_indices: Option<Vec<u8>>,
        pub vertex_group_weights: Option<Vec<f32>>,
    }

    impl Into<SingleIndexedVertexAttributes> for TodoDeleteMeSingleConverter {
        fn into(self) -> SingleIndexedVertexAttributes {
            let bones = match self.bone_influences_per_vertex.as_ref() {
                None => None,
                Some(b) => {
                    let b = match b {
                        BoneInfluencesPerVertex::NonUniform(_) => unreachable!(),
                        BoneInfluencesPerVertex::Uniform(b) => *b as _,
                    };

                    Some((
                        BoneAttributes {
                            bone_influencers: (self.vertex_group_indices.unwrap(), b).into(),
                            bone_weights: (self.vertex_group_weights.unwrap(), b).into(),
                        },
                        b,
                    ))
                }
            };

            SingleIndexedVertexAttributes {
                indices: self.vertex_position_indices,
                vertices: make_vertices(
                    self.vertex_positions,
                    Some(self.vertex_normals),
                    self.vertex_uvs,
                    self.tangents,
                    bones,
                ),
            }
        }
    }
}
