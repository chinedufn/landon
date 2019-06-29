//! Utility functions for getting and setting data for individual vertices.

use crate::BlenderMesh;

impl BlenderMesh {
    /// Given a vertex position idx, get the corresponding vertex position
    pub(crate) fn vertex_pos_at_idx(&self, vertex_pos_idx: u16) -> (f32, f32, f32) {
        let vertex_pos_idx = vertex_pos_idx as usize;

        (
            self.vertex_positions[vertex_pos_idx * 3],
            self.vertex_positions[vertex_pos_idx * 3 + 1],
            self.vertex_positions[vertex_pos_idx * 3 + 2],
        )
    }
}

impl BlenderMesh {
    /// Given a vertex normal idx, get the corresponding vertex normal
    pub(crate) fn vertex_normal_at_idx(&self, vertex_normal_idx: u16) -> (f32, f32, f32) {
        let vertex_normal_idx = vertex_normal_idx as usize;

        (
            self.vertex_normals[vertex_normal_idx * 3],
            self.vertex_normals[vertex_normal_idx * 3 + 1],
            self.vertex_normals[vertex_normal_idx * 3 + 2],
        )
    }
}

impl BlenderMesh {
    /// Given a vertex uv idx, get the corresponding vertex uv
    pub(crate) fn vertex_uv_at_idx(&self, vertex_uv_idx: u16) -> (f32, f32) {
        let vertex_uv_idx = vertex_uv_idx as usize;
        let uvs = self.vertex_uvs.as_ref().unwrap();

        (uvs[vertex_uv_idx * 2], uvs[vertex_uv_idx * 2 + 1])
    }
}
