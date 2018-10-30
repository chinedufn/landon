use nalgebra::Point3;

/// The bounding box that encompasses a mesh. This will usually come from Blender as a z_up
/// coordinate system bounding box that you'll later convert to be y_up.
///
/// If your mesh is parented to an armature then this is the bounding box of your mesh in its
/// bind pose.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    /// The corner with the lowest x, y and z values
    pub min_corner: Point3<f32>,
    /// The corner with the greatest x, y and z values
    pub max_corner: Point3<f32>,
}

#[cfg(test)]
impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            min_corner: Point3::new(0.0, 0.0, 0.0),
            max_corner: Point3::new(0.0, 0.0, 0.0),
        }
    }
}
