use crate::vertex_attributes::IndexedAttribute;
use crate::{
    BlenderMesh, BoundingBox, MaterialInput, MultiIndexedVertexAttributes, PrincipledBSDF,
    VertexAttribute,
};
use std::collections::HashMap;

impl BlenderMesh {
    /// Create a default Blender cube with uniform PBR texture inputs.
    ///
    /// A 2x2x2 cube centered about the origin.
    pub fn pbr_cube_without_textures() -> Self {
        let mut materials = HashMap::with_capacity(1);
        materials.insert(
            "Default".to_string(),
            PrincipledBSDF {
                base_color: MaterialInput::Uniform([0.4, 0.5, 0.6]),
                roughness: MaterialInput::Uniform(0.2),
                metallic: MaterialInput::Uniform(0.3),
                normal_map: None,
            },
        );

        let multi_indexed_vertex_attributes = MultiIndexedVertexAttributes {
            vertices_in_each_face: vec![4, 4, 4, 4, 4, 4],
            positions: IndexedAttribute::new(
                vec![
                    0, 1, 2, 3, 4, 7, 6, 5, 0, 4, 5, 1, 1, 5, 6, 2, 2, 6, 7, 3, 4, 0, 3, 7,
                ],
                VertexAttribute::new(
                    vec![
                        1., 1., -1., 1., -1., -1., -1., -1., -1., -1., 1., -1., 1., 1., 1., 1.,
                        -1., 1., -1., -1., 1., -1., 1., 1.,
                    ],
                    3,
                )
                .unwrap(),
            ),
            normals: Some(IndexedAttribute::new(
                vec![
                    0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5,
                ],
                VertexAttribute::new(
                    vec![
                        0., 0., -1., 0., 0., 1., 1., 0., 0., 0., -1., 0., -1., 0., 0., 0., 1., 0.,
                    ],
                    3,
                )
                .unwrap(),
            )),
            uvs: None,
            bone_influences: None,
        };

        Self {
            name: "CubeWithoutTextures".to_string(),
            armature_name: None,
            bounding_box: BoundingBox {
                min_corner: [-1.; 3].into(),
                max_corner: [1.; 3].into(),
            },
            multi_indexed_vertex_attributes,
            materials,
            custom_properties: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CreateSingleIndexConfig;

    /// Verify that we can combine the positions and normals into a single buffer
    #[test]
    fn generate_vertex_buffer() {
        let mut mesh = BlenderMesh::pbr_cube_without_textures();
        mesh.combine_vertex_indices(&CreateSingleIndexConfig {
            bone_influences_per_vertex: None,
            calculate_face_tangents: false,
        });
    }
}
