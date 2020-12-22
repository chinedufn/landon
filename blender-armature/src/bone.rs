use nalgebra::{DualQuaternion, Matrix4};

/// A bone in an armature. Can either be a dual quaternion or a matrix. When you export bones
/// from Blender they come as matrices - BlenderArmature lets you convert them into dual
/// quaternions which are usually more favorable for when implementing skeletal animation with
/// rigid transformations.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Bone {
    /// A transform represented as a matrix
    Matrix(Matrix4<f32>),
    /// A rigid transform represented as a dual quaternion
    DualQuat(DualQuaternion<f32>),
}

impl Bone {
    /// Get this bone's transform relative to some parent bone.
    ///
    /// You could typically use this when both bones are in world space.
    ///
    /// ```
    /// # use blender_armature::Bone;
    /// use nalgebra::{Matrix4, Vector3, Point3};
    ///
    /// let axis_angle = Vector3::default();
    ///
    /// let parent = Bone::Matrix(Matrix4::new_rotation_wrt_point(
    ///   axis_angle,
    ///   Point3::new(4., 5., 6.)
    /// ));
    ///
    /// let child = Bone::Matrix(Matrix4::new_rotation_wrt_point(
    ///   axis_angle,
    ///   Point3::new(44., 55., 66.)
    /// ));
    ///
    /// let expected = Bone::Matrix(Matrix4::new_rotation_wrt_point(
    ///   axis_angle,
    ///   Point3::new(40., 50., 60.)
    /// ));
    ///
    /// assert_eq!(child.relative_to_parent(parent), expected);
    /// ```
    pub fn relative_to_parent(&self, parent_bone: Bone) -> Bone {
        match (self, parent_bone) {
            (Bone::Matrix(child), Bone::Matrix(parent)) => {
                //
                Bone::Matrix(parent.try_inverse().unwrap() * child)
            }
            _ => unimplemented!(),
        }
    }
}
