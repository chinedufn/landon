use nalgebra::Vector3;

/// The bounding box that encompasses a mesh. This will usually come from Blender as a z_up
/// coordinate system bounding box that you'll later convert to be y_up.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    /// If you're staring into the scene, this is the bottom left corner of the face that is
    /// closest to you.
    lower_left_front: Vector3<f32>,
    /// If you're staring into the scene, this is the top right corner of the face that is
    /// farthest from you.
    upper_right_back: Vector3<f32>,
}
