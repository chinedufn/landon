//! Data structures and methods for dealing with armatures.
//!
//! @see https://docs.blender.org/manual/en/dev/modeling/armature/introduction.html - Armature Introduction

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

use crate::serde::serialize_hashmap_deterministic;

pub use self::action::*;
pub use self::bone::*;
pub use self::coordinate_system::*;
pub use self::export::*;
pub use self::interpolate::*;
use std::borrow::Borrow;
use std::hash::Hash;

mod action;
mod bone;
mod convert;
mod coordinate_system;
mod export;
mod interpolate;
mod serde;

#[cfg(test)]
mod test_util;

/// Something went wrong in the Blender child process that was trying to parse your armature data.
#[derive(Debug, thiserror::Error)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export armature from a `.blend` file.
    #[error(
        "There was an issue while exporting armature: Blender stderr output: {}",
        _0
    )]
    Stderr(String),
}

/// All of the data about a Blender armature that we've exported from Blender.
/// A BlenderArmature should have all of the data that you need to implement skeletal
/// animation.
///
/// If you have other needs, such as a way to know the model space position of any bone at any
/// time so that you can, say, render a baseball in on top of your hand bone.. Open an issue.
/// (I plan to support this specific example in the future)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))]
// TODO: BlenderArmature<T: Bone> for DQ and matrix
pub struct BlenderArmature {
    name: String,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    joint_indices: HashMap<String, u8>,
    bone_child_to_parent: HashMap<u8, u8>,
    inverse_bind_poses: Vec<Bone>,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    bone_space_actions: HashMap<String, Action>,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    bone_groups: HashMap<String, Vec<u8>>,
    #[serde(default)]
    coordinate_system: CoordinateSystem,
}

impl BlenderArmature {
    /// The name of the armature
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Set the name of the armature.
    ///
    /// # Example
    ///
    /// ```
    /// # use blender_armature::BlenderArmature;
    /// let mut armature = BlenderArmature::default();
    /// armature.set_name("Some Name".to_string());
    ///
    /// assert_eq!(armature.name(), "Some Name");
    /// ```
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Blender [bone groups]
    ///
    /// Maps bone group name to a vector of the bones indices that are in that bone group.
    ///
    /// ```rust
    /// # use blender_armature::{Action, BlenderArmature, FrameOffset, SampleDesc, JointIndicesRef};
    /// # use std::time::Duration;
    ///
    /// let armature = create_blender_armature();
    ///
    /// let joint_indices = armature.bone_groups().get("My bone group").unwrap();
    ///
    /// let sample_desc = SampleDesc {
    ///     frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
    ///         Duration::from_secs(2),
    ///         24,
    ///     ),
    ///     should_loop: false
    /// };
    ///
    /// let _bones = armature.interpolate_bones(
    ///     "SomeAction",
    ///     JointIndicesRef::Some(joint_indices),
    ///     sample_desc
    /// );
    ///
    /// # fn create_blender_armature() -> BlenderArmature {
    /// #   let mut  b = BlenderArmature::default();
    /// #   b.insert_bone_space_action("SomeAction".to_string(), Action::new());
    /// #   b.create_bone_group("My bone group".to_string(), vec![]);
    /// #   b
    /// # }
    /// ```
    ///
    /// [bone groups]: https://docs.blender.org/manual/en/latest/animation/armatures/properties/bone_groups.html
    pub fn bone_groups(&self) -> &HashMap<String, Vec<u8>> {
        &self.bone_groups
    }

    /// Create a new bone group
    pub fn create_bone_group(&mut self, name: String, joint_indices: Vec<u8>) {
        self.bone_groups.insert(name, joint_indices);
    }

    /// Get a bone's index into the various Vec<Bone> data structures that hold bone data.
    ///
    /// # Example
    ///
    /// ```
    /// use blender_armature::BlenderArmature;
    /// let mut armature = BlenderArmature::default();
    ///
    /// armature.insert_joint_index("Spine".to_string(), 0);
    ///
    /// assert_eq!(armature.joint_indices().len(), 1);
    /// ```
    pub fn joint_indices(&self) -> &HashMap<String, u8> {
        &self.joint_indices
    }

    /// Set a bone's index into the various Vec<Bone> data structures that hold bone data.
    ///
    /// # Example
    ///
    /// ```
    /// use blender_armature::BlenderArmature;
    /// let mut armature = BlenderArmature::default();
    ///
    /// armature.insert_joint_index("Spine".to_string(), 0);
    /// armature.insert_joint_index("UpperArm".to_string(), 2);
    ///
    /// assert_eq!(armature.joint_indices().len(), 2);
    /// ```
    pub fn insert_joint_index(&mut self, joint_name: String, joint_idx: u8) {
        self.joint_indices.insert(joint_name, joint_idx);
    }

    /// Every bone's inverse bind pose.
    ///
    /// # From Blender
    /// When exporting from Blender these include the armature's world space matrix.
    ///
    /// So, effectively these are `(armature_world_space_matrix * bone_bind_pose).inverse()`
    pub fn inverse_bind_poses(&self) -> &Vec<Bone> {
        &self.inverse_bind_poses
    }

    /// Set the inverse bind poses.
    pub fn set_inverse_bind_poses(&mut self, poses: Vec<Bone>) {
        self.inverse_bind_poses = poses;
    }

    /// All of the actions defined on the armature, keyed by action name.
    ///
    /// FIXME: Rename to `bone_local_space_actions`
    pub fn bone_space_actions(&self) -> &HashMap<String, Action> {
        &self.bone_space_actions
    }

    /// Insert an action into the map of actions.
    pub fn insert_bone_space_action(&mut self, name: String, action: Action) {
        self.bone_space_actions.insert(name, action);
    }

    /// Remove an action from the map.
    pub fn remove_bone_space_action<Q>(&mut self, name: &Q) -> Option<Action>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.bone_space_actions.remove(name)
    }

    /// A map of a bone chil to its parent
    ///
    /// If a bone is not stored in this map then it does not have a parent.
    pub fn bone_child_to_parent(&self) -> &HashMap<u8, u8> {
        &self.bone_child_to_parent
    }

    /// # Example
    ///
    /// ```
    /// # use blender_armature::BlenderArmature;
    /// let mut armature = BlenderArmature::default();
    ///
    /// let child_idx = 4;
    /// let parent_idx = 2;
    ///
    /// armature.insert_joint_index("UpperArm".to_string(), parent_idx);
    /// armature.insert_joint_index("Lower Arm".to_string(), child_idx);
    ///
    /// armature.insert_child_to_parent(child_idx, parent_idx);
    /// ```
    pub fn insert_child_to_parent(&mut self, child: u8, parent: u8) {
        self.bone_child_to_parent.insert(child, parent);
    }
}

/// The pose bones at an individual keyframe time
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default, Clone))]
pub struct Keyframe {
    frame: u16,
    bones: Vec<Bone>,
}

impl Keyframe {
    #[allow(missing_docs)]
    pub fn new(frame: u16, bones: Vec<Bone>) -> Self {
        Keyframe { frame, bones }
    }

    /// All of the bones for this keyframe.
    pub fn bones(&self) -> &Vec<Bone> {
        &self.bones
    }

    /// All of the bones for this keyframe.
    pub fn bones_mut(&mut self) -> &mut Vec<Bone> {
        &mut self.bones
    }

    /// The frame number
    pub fn frame(&self) -> u16 {
        self.frame
    }
}

// TODO: These methods can be abstracted into calling a method that takes a callback
impl BlenderArmature {
    /// Tranpose all of the bone matrices in our armature's action keyframes.
    /// Blender uses row major matrices, but OpenGL uses column major matrices so you'll
    /// usually want to transpose your matrices before using them.
    pub fn transpose_actions(&mut self) {
        for (_name, action) in self.bone_space_actions.iter_mut() {
            for (_bone_idx, keyframes) in action.keyframes_mut().iter_mut() {
                for bone in keyframes.iter_mut() {
                    bone.bone_mut().transpose();
                }
            }
        }

        for bone in self.inverse_bind_poses.iter_mut() {
            bone.transpose();
        }
    }
}

impl BlenderArmature {
    /// Convert your action matrices into dual quaternions so that you can implement
    /// dual quaternion linear blending.
    pub fn matrices_to_dual_quats(&mut self) {
        for (_, keyframes) in self.bone_space_actions.iter_mut() {
            for (bone_idx, keyframes) in keyframes.keyframes_mut().iter_mut() {
                for bone_keyframe in keyframes.iter_mut() {
                    bone_keyframe
                        .set_bone(BlenderArmature::matrix_to_dual_quat(&bone_keyframe.bone()));
                }
            }
        }

        for bone in self.inverse_bind_poses.iter_mut() {
            *bone = BlenderArmature::matrix_to_dual_quat(bone);
        }
    }
}

impl Bone {
    fn transpose(&mut self) {
        match self {
            Bone::Matrix(ref mut matrix) => {
                matrix.transpose_mut();
            }
            Bone::DualQuat(_) => unimplemented!(),
        };
    }

    // DELETE ME
    fn multiply(&mut self, rhs: Bone) {
        match self {
            Bone::Matrix(lhs_matrix) => match rhs {
                Bone::Matrix(rhs_matrix) => {
                    //
                    *self = Bone::Matrix(rhs_matrix * *lhs_matrix)
                }
                Bone::DualQuat(_) => {}
            },
            Bone::DualQuat(_) => {}
        };
    }
}

// DELETE ME
impl BlenderArmature {
    /// Iterate over all of the action bones and apply and multiply in the inverse bind pose.
    ///
    /// TODO: another function to apply bind shape matrix? Most armatures seem to export an identity
    ///  bind shape matrix but that might not be the same for every armature.
    ///
    /// TODO: Do not mutate the matrices and instead just return the new values and let the caller
    ///  handle caching them? Would mean less moving parts in our data structures and you always
    ///  know exactly what you are getting. Right now you have no way actions of knowing whether or
    ///  not actions have their bind poses pre-multiplied in.
    pub fn apply_inverse_bind_poses(&mut self) {
        for (_name, action) in self.bone_space_actions.iter_mut() {
            for (bone_idx, keyframe) in action.keyframes_mut().iter_mut() {
                for (index, bone) in keyframe.iter_mut().enumerate() {
                    bone.bone_mut()
                        .multiply(self.inverse_bind_poses[*bone_idx as usize]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::tests::dq_to_bone;
    use crate::test_util::action_with_keyframes;
    use nalgebra::Matrix4;

    #[test]
    fn convert_actions_to_dual_quats() {
        let mut keyframes = vec![];
        keyframes.push(BoneKeyframe::new(
            1,
            Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ])),
        ));

        let mut start_armature = BlenderArmature {
            bone_space_actions: action_with_keyframes(keyframes),
            ..BlenderArmature::default()
        };

        start_armature.matrices_to_dual_quats();

        let mut new_keyframes = vec![];
        new_keyframes.push(BoneKeyframe::new(
            1,
            dq_to_bone([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ));

        let expected_armature = BlenderArmature {
            bone_space_actions: action_with_keyframes(new_keyframes),
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }

    // TODO: Function to return these start_actions that we keep using
    #[test]
    fn transpose_actions() {
        let keyframes = vec![BoneKeyframe::new(
            1,
            Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 5.0, 1.0,
            ])),
        )];

        let mut start_armature = BlenderArmature {
            bone_space_actions: action_with_keyframes(keyframes),
            ..BlenderArmature::default()
        };

        start_armature.transpose_actions();

        let new_keyframes = vec![BoneKeyframe::new(
            1,
            Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0, 0.0, 0.0, 0.0, 1.0,
            ])),
        )];

        let expected_armature = BlenderArmature {
            bone_space_actions: action_with_keyframes(new_keyframes),
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }
}
