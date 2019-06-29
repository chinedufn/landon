use crate::concat_vecs;

/// Create a 3 dimensional vector with all three values the same.
/// Useful for quickly generating some fake vertex data.
/// v(0.0) -> vec![0.0, 0.0, 0.0]
///
/// TODO: Rename to v3
pub fn v(val: u8) -> Vec<f32> {
    vec![val as f32, val as f32, val as f32]
}

/// A vector with 2 items
pub fn v2(val: u8) -> Vec<f32> {
    vec![val as f32, val as f32]
}

/// Creates 4 sets of two numbers.
/// Useful for generating 4 uvs for a quad
pub fn v2_x4(vert1: u8, vert2: u8, vert3: u8, vert4: u8) -> Vec<f32> {
    concat_vecs!(v2(vert1), v2(vert2), v2(vert3), v2(vert4))
}

/// Create 4 sets of three numbers.
/// Useful for generating 4 normals or positions for a quad.
pub fn v3_x4(v1: u8, v2: u8, v3: u8, v4: u8) -> Vec<f32> {
    concat_vecs!(v(v1), v(v2), v(v3), v(v4))
}

/// Create 3 sets of three numbers.
/// Useful for generating 3 normals for a triangle.
pub fn v3_x3(v1: u8, v2: u8, v3: u8) -> Vec<f32> {
    concat_vecs!(v(v1), v(v2), v(v3))
}
