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
        for vert_num in 0..(self.vertex_positions.len() / 3) {
            let y_index = vert_num * 3 + 1;
            let z_index = y_index + 1;

            let new_z = -self.vertex_positions[y_index];
            self.vertex_positions[y_index] = self.vertex_positions[z_index];
            self.vertex_positions[z_index] = new_z;
        }

        for vert_num in 0..(self.vertex_normals.len() / 3) {
            let y_index = vert_num * 3 + 1;
            let z_index = y_index + 1;

            let new_z = -self.vertex_normals[y_index];
            self.vertex_normals[y_index] = self.vertex_normals[z_index];
            self.vertex_normals[z_index] = new_z;
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
    use nalgebra::Point3;

    #[test]
    fn z_up_to_y_up() {
        let mut start_mesh = BlenderMesh {
            vertex_positions: vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
            vertex_normals: vec![0.0, 1.0, 2.0],
            bounding_box: BoundingBox {
                min_corner: Point3::new(1.0, 2.0, 3.0),
                max_corner: Point3::new(5.0, 6.0, 7.0),
            },
            ..BlenderMesh::default()
        };

        start_mesh.y_up();
        let y_up_mesh = start_mesh;

        let expected_mesh = BlenderMesh {
            vertex_positions: vec![0.0, 2.0, -1.0, 0.0, 2.0, -1.0],
            vertex_normals: vec![0.0, 2.0, -1.0],
            bounding_box: BoundingBox {
                min_corner: Point3::new(1.0, 3.0, -2.0),
                max_corner: Point3::new(5.0, 7.0, -6.0),
            },
            ..BlenderMesh::default()
        };

        assert_eq!(y_up_mesh, expected_mesh);
    }
}
