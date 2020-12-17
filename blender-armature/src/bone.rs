use nalgebra::{DualQuaternion, Matrix4};

/// A bone in an armature. Can either be a dual quaternion or a matrix. When you export bones
/// from Blender they come as matrices - BlenderArmature lets you convert them into dual
/// quaternions which are usually more favorable for when implementing skeletal animation.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Bone {
    /// TODO: Only support dual quarternions? We could use a custom derive to automatically convert
    ///  [f32;16] matrices into [f32;8] dual quaternion (to avoid needing to get dual quat logic
    ///  working in the python export script).
    ///  We could also just write our export script in Rust and not use a custom deserialize
    ///  Better yet ... just store both the matrix and the dual quaternion representation so that
    ///  we can use either one depending on the scenario.
    ///  If memory ever became an issue we could put matrices behind a feature flag.
    Matrix(Matrix4<f32>),
    /// Rotation:     [w, x, y, z]
    /// Translation:  [w, x, y, z]
    DualQuat(DualQuaternion<f32>),
}
