use crate::BlenderMesh;

impl BlenderMesh {
    /// When exporting a mesh from Blender, faces will usually have 4 vertices (quad) but some
    /// faces might have 3 (triangle).
    ///
    /// We read `self.num_vertices_in_each_face` to check how
    /// many vertices each face has.
    ///
    /// If a face has 4 vertices we convert it into two triangles, each with 3 vertices.
    ///
    /// # Panics
    ///
    /// Panics if a face has more than 4 vertices. In the future we might support 5+ vertices,
    /// but I haven't run into that yet. Not even sure if Blender can have faces with 5 vertices..
    pub fn triangulate(&mut self) {
        let mut triangulated_position_indices = vec![];
        let mut triangulated_face_vertex_counts = vec![];

        let mut face_pointer = 0;

        let multi = &mut self.multi_indexed_vertex_attributes;

        for num_verts_in_face in multi.vertices_in_each_face.iter() {
            triangulated_position_indices.push(multi.positions.indices[face_pointer]);
            triangulated_position_indices.push(multi.positions.indices[face_pointer + 1]);
            triangulated_position_indices.push(multi.positions.indices[face_pointer + 2]);

            triangulated_face_vertex_counts.push(3);

            match num_verts_in_face {
                &3 => {}
                &4 => {
                    triangulated_position_indices.push(multi.positions.indices[face_pointer]);
                    triangulated_position_indices.push(multi.positions.indices[face_pointer + 2]);
                    triangulated_position_indices.push(multi.positions.indices[face_pointer + 3]);

                    triangulated_face_vertex_counts.push(3);
                }
                _ => {
                    panic!("blender-mesh currently only supports triangulating faces with 3 or 4 vertices");
                }
            };

            face_pointer += *num_verts_in_face as usize;
        }

        multi.positions.indices = triangulated_position_indices;
        multi.vertices_in_each_face = triangulated_face_vertex_counts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertex_attributes::{
        IndexedAttribute, MultiIndexedVertexAttributes, VertexAttribute,
    };

    #[test]
    fn triangulate_faces() {
        let mut start_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: IndexedAttribute {
                    indices: vec![0, 1, 2, 3, 4, 5, 6, 7],
                    attribute: VertexAttribute::default(),
                },
                vertices_in_each_face: vec![4, 4],
                ..MultiIndexedVertexAttributes::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        start_mesh.triangulate();
        let triangulated_mesh = start_mesh;

        let expected_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: IndexedAttribute {
                    indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
                    attribute: VertexAttribute::default(),
                },
                vertices_in_each_face: vec![3, 3, 3, 3],
                ..MultiIndexedVertexAttributes::default()
            }
            .into(),
            ..BlenderMesh::default()
        };

        assert_eq!(triangulated_mesh, expected_mesh);
    }
}
