use crate::{BlenderArmature, Bone};
use std::borrow::Borrow;
use std::hash::Hash;

impl BlenderArmature {
    /// Get the bind pose position of a bone using its name
    ///
    /// TODO: Multiply by the parent matrices first.
    ///  The method currently only works for bones that do not have
    ///  parents such as those from blender-iks-to-fks.
    ///
    /// # Panics
    ///
    /// Panics if there is no bone with the provided name
    pub fn bone_model_space_position_with_name<Q: ?Sized>(&self, name: &Q) -> [f32; 3]
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = self.joint_indices.get(name).unwrap();

        self.bone_model_space_bind_position(*idx)
    }

    /// Get the bind pose position of a bone using its bone index
    ///
    /// TODO: Multiply by the parent matrices first.
    ///  The method currently only works for bones that do not have
    ///  parents such as those from blender-iks-to-fks.
    ///
    /// # Panics
    ///
    /// Panics if there iss no bone at the provided index
    pub fn bone_model_space_bind_position(&self, bone_idx: u8) -> [f32; 3] {
        match self.inverse_bind_poses[bone_idx as usize] {
            Bone::Matrix(_) => unimplemented!(),
            Bone::DualQuat(dq) => [-dq[5], -dq[6], -dq[7]],
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Bone;

    /// Get the bind position using the bone's index
    #[test]
    fn bind_pos_with_idx() {
        let mut armature = BlenderArmature::default();
        armature.inverse_bind_poses.push(bone());

        assert_eq!(armature.bone_model_space_bind_position(0), [-6., -7., -8.]);
    }

    fn bone() -> Bone {
        Bone::DualQuat([1., 2., 3., 4., 5., 6., 7., 8.])
    }
}
