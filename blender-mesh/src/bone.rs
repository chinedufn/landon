use crate::vertex_attributes::{MultiIndexedVertexAttributes, VertexBoneInfluences};

/// The number of bones that influence each uniform.
///
/// When exported from Blender this is non uniform, but becomes uniform when
/// we call `.set_groups_per_vertex` to make every vertex have the same number
/// of influences.
///
/// TODO: Remove this and use VertexAttribute with something like attribute_size: Varies(vec![])
/// this allows us to handle all attributes the same way.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum BoneInfluencesPerVertex {
    NonUniform(Vec<u8>),
    Uniform(u8),
}

impl Default for BoneInfluencesPerVertex {
    fn default() -> Self {
        BoneInfluencesPerVertex::Uniform(0)
    }
}

impl From<Vec<u8>> for BoneInfluencesPerVertex {
    fn from(influences: Vec<u8>) -> Self {
        BoneInfluencesPerVertex::NonUniform(influences)
    }
}

impl MultiIndexedVertexAttributes {
    /// Different vertices might have different numbers of bones that influence them.
    /// A vertex near the shoulder might be influenced by the neck and upper arm and sternum,
    /// while a vertex in a toe might only be influenced by a toe bone.
    ///
    /// When passing data to the GPU, each vertex needs the same number of bone attributes, so
    /// we must add/remove bones from each vertex to get them equal.
    ///
    /// Say we're setting 3 groups per vertex:
    ///  - If a vertex has one vertex group (bone) we will create two fake bones with 0.0 weight.
    ///  - If a vertex has 5 bones we'll remove the one with the smallest weighting (influence).
    ///
    /// TODO: I wrote landon when I was a Rust noob and so a lot of things are just wrong. This
    /// method doesn't work if you try to set the bone influences to two different constants in
    /// a row - for example.
    ///       Need to eventually rewrite a lot of the library with TDD - but we can clean up over
    ///       time.
    pub(crate) fn set_bone_influences_per_vertex(&mut self, count: u8) {
        let mut normalized_group_indices = vec![];
        let mut normalized_group_weights = vec![];

        let mut current_index: u32 = 0;

        // TODO: Error handling
        if self.bone_influences.is_none() {
            return;
        }

        {
            let VertexBoneInfluences {
                bones_per_vertex,
                bone_indices,
                bone_weights,
            } = self.bone_influences.as_mut().unwrap();

            if let BoneInfluencesPerVertex::NonUniform(bone_influences_per_vertex) =
                &bones_per_vertex
            {
                for group_count in bone_influences_per_vertex.iter() {
                    let mut vertex_indices = vec![];
                    let mut vertex_weights = vec![];

                    for index in current_index..(current_index + *group_count as u32) {
                        vertex_indices.push(index);
                        vertex_weights.push(bone_weights[index as usize]);
                    }

                    vertex_weights.sort_by(|a, b| b.partial_cmp(a).unwrap());
                    vertex_indices.sort_by(|a, b| {
                        bone_weights[*b as usize]
                            .partial_cmp(&bone_weights[*a as usize])
                            .unwrap()
                    });

                    let mut vertex_indices: Vec<u8> = vertex_indices
                        .iter()
                        .map(|i| bone_indices[*i as usize])
                        .collect();

                    vertex_indices.resize(count as usize, 0);
                    vertex_weights.resize(count as usize, 0.0);

                    normalized_group_indices.append(&mut vertex_indices);
                    normalized_group_weights.append(&mut vertex_weights);

                    current_index += *group_count as u32;
                }
            }
        }

        let mut bone_influences = self.bone_influences.as_mut().unwrap();

        bone_influences.bones_per_vertex = BoneInfluencesPerVertex::Uniform(count);
        bone_influences.bone_indices = normalized_group_indices;
        bone_influences.bone_weights = normalized_group_weights;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combine_indices::tests::TodoDeleteMeMultiConverter;
    use crate::BlenderMesh;

    #[test]
    fn set_joints_per_vert() {
        let mut start_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_group_indices: Some(vec![0, 2, 3, 4, 0, 1, 3, 2]),
                bone_influences_per_vertex: Some(vec![1, 3, 4].into()),
                vertex_group_weights: Some(vec![1.0, 0.5, 0.2, 0.3, 0.6, 0.15, 0.1, 0.15]),
                ..TodoDeleteMeMultiConverter::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        start_mesh
            .multi_indexed_vertex_attributes
            .set_bone_influences_per_vertex(3);
        let three_joints_per_vert = start_mesh;

        let expected_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: TodoDeleteMeMultiConverter {
                vertex_group_indices: Some(vec![0, 0, 0, 2, 4, 3, 0, 1, 2]),
                bone_influences_per_vertex: Some(BoneInfluencesPerVertex::Uniform(3)),
                vertex_group_weights: Some(vec![1.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.6, 0.15, 0.15]),
                ..TodoDeleteMeMultiConverter::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        assert_eq!(three_joints_per_vert, expected_mesh);
    }
}
