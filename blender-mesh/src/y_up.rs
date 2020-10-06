use crate::BlenderMesh;

static Y: usize = 1;
static Z: usize = 2;

impl BlenderMesh {
    /// Blender meshes get exported with a Z up coordinate system.
    /// Here we flip our coordinate system to be y up
    ///
    /// @see https://gamedev.stackexchange.com/a/7932
    ///
    /// TODO: When we have bone data we'll need to change them to port change-mat4-coordinate-system
    /// into here.
    /// https://github.com/chinedufn/change-mat4-coordinate-system/blob/master/change-mat4-coordinate-system.js
    pub fn y_up(&mut self) {
        let vertex_attribs = &mut self.multi_indexed_vertex_attributes;

        let positions = &mut vertex_attribs.positions.attribute.data;

        let mut normals = match &mut vertex_attribs.normals {
            None => None,
            Some(normals) => Some(&mut normals.attribute.data),
        };

        let convert = |vert_num: usize, data: &mut Vec<f32>| {
            let y_index = vert_num * 3 + 1;
            let z_index = y_index + 1;

            let new_z = -data[y_index];
            data[y_index] = data[z_index];
            data[z_index] = new_z;
        };

        for vert_num in 0..positions.len() / 3 {
            convert(vert_num, positions);
        }

        if let Some(normals) = normals.as_mut() {
            for vert_num in 0..normals.len() / 3 {
                convert(vert_num, normals);
            }
        }

        let new_z = -self.bounding_box.min_corner[Y];
        self.bounding_box.min_corner[Y] = self.bounding_box.min_corner[Z];
        self.bounding_box.min_corner[Z] = new_z;

        let new_z = -self.bounding_box.max_corner[Y];
        self.bounding_box.max_corner[Y] = self.bounding_box.max_corner[Z];
        self.bounding_box.max_corner[Z] = new_z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounding_box::BoundingBox;
    use crate::indexed;
    use crate::vertex_attributes::MultiIndexedVertexAttributes;
    use nalgebra::Point3;

    #[test]
    fn z_up_to_y_up() {
        let mut start_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: indexed((vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0], 3).into()),
                normals: Some(indexed((vec![0.0, 1.0, 2.0], 3).into())),
                ..MultiIndexedVertexAttributes::default()
            },
            bounding_box: BoundingBox {
                min_corner: Point3::new(1.0, 2.0, 3.0),
                max_corner: Point3::new(5.0, 6.0, 7.0),
            },
            ..BlenderMesh::default()
        };

        start_mesh.y_up();
        let y_up_mesh = start_mesh;

        let expected_mesh = BlenderMesh {
            multi_indexed_vertex_attributes: MultiIndexedVertexAttributes {
                positions: indexed((vec![0.0, 2.0, -1.0, 0.0, 2.0, -1.0], 3).into()),
                normals: Some(indexed((vec![0.0, 2.0, -1.0], 3).into())),
                ..MultiIndexedVertexAttributes::default()
            },
            bounding_box: BoundingBox {
                min_corner: Point3::new(1.0, 3.0, -2.0),
                max_corner: Point3::new(5.0, 7.0, -6.0),
            },
            ..BlenderMesh::default()
        };

        assert_eq!(y_up_mesh, expected_mesh);
    }
}
