use nalgebra::Vector3;

/// The bounding box that encompasses a mesh. This will usually come from Blender as a z_up
/// coordinate system bounding box that you'll later convert to be y_up.
///
/// If your mesh is parented to an armature then this is the bounding box of your mesh in its
/// bind pose.
///
/// TODO: Rename to `smallest_corner` and `largest_corner`. Representing lowest x/y/z and largest
/// x/y/z
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    /// If you're staring into the scene, this is the bottom left corner of the face that is
    /// closest to you.
    pub lower_left_front: Vector3<f32>,
    /// If you're staring into the scene, this is the top right corner of the face that is
    /// farthest from you.
    pub upper_right_back: Vector3<f32>,
}

#[cfg(test)]
impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            lower_left_front: Vector3::new(0.0, 0.0, 0.0),
            upper_right_back: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}
