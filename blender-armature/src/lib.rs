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
use nalgebra::Matrix4;

mod action;
mod bone;
mod convert;
mod coordinate_system;
mod export;
mod interpolate;
mod serde;

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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct BlenderArmature {
    name: String,
    world_space_matrix: Matrix4<f32>,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    joint_indices: HashMap<String, u8>,
    bone_parents: HashMap<u8, Option<u8>>,
    inverse_bind_poses: Vec<Bone>,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    actions: HashMap<String, Action>,
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
    /// #   b.actions_mut().insert("SomeAction".to_string(), Action::new(vec![]));
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
    pub fn joint_indices(&self) -> &HashMap<String, u8> {
        &self.joint_indices
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

    /// See [`BlenderArmature.inverse_bind_poses()`]
    pub fn inverse_bind_poses_mut(&mut self) -> &mut Vec<Bone> {
        &mut self.inverse_bind_poses
    }

    /// All of the actions defined on the armature, keyed by action name.
    ///
    /// # From Blender
    ///
    /// When exporting from Blender these are the pose bone's armature space matrix.
    ///
    /// Note that this means that these bones are not relative to their parent's.
    ///
    /// To get a pose bone relative to it's parent use [`Bone.relative_to()`]
    pub fn actions(&self) -> &HashMap<String, Action> {
        &self.actions
    }

    /// See [`BlenderArmature.method#actions`]
    pub fn actions_mut(&mut self) -> &mut HashMap<String, Action> {
        &mut self.actions
    }

    /// The transformation matrix for the armature within the world that it was defined in
    /// (i.e. a Blender scene).
    ///
    /// If you apply location, rotation and scale to the armature in Blender then this will be an
    /// identity matrix.
    ///
    /// https://docs.blender.org/api/current/bpy.types.Object.html#bpy.types.Object.matrix_world
    pub fn world_space_matrix(&self) -> Matrix4<f32> {
        self.world_space_matrix
    }

    /// The parent of each bone
    pub fn bone_parents(&self) -> &HashMap<u8, Option<u8>> {
        &self.bone_parents
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
        for (_name, action) in self.actions.iter_mut() {
            for keyframe in action.keyframes_mut().iter_mut() {
                for (_index, bone) in keyframe.bones.iter_mut().enumerate() {
                    bone.transpose();
                }
            }
        }
    }
}

impl BlenderArmature {
    /// Convert your action matrices into dual quaternions so that you can implement
    /// dual quaternion linear blending.
    pub fn matrices_to_dual_quats(&mut self) {
        for (_, keyframes) in self.actions.iter_mut() {
            for keyframe in keyframes.keyframes_mut().iter_mut() {
                for bone in keyframe.bones.iter_mut() {
                    *bone = BlenderArmature::matrix_to_dual_quat(bone);
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
}

impl Default for BlenderArmature {
    fn default() -> Self {
        BlenderArmature {
            name: "".to_string(),
            world_space_matrix: Matrix4::identity(),
            joint_indices: Default::default(),
            bone_parents: Default::default(),
            inverse_bind_poses: vec![],
            actions: Default::default(),
            bone_groups: Default::default(),
            coordinate_system: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::tests::dq_to_bone;

    #[test]
    fn convert_actions_to_dual_quats() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]))],
        });
        start_actions.insert("Fly".to_string(), Action::new(keyframes));

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            ..BlenderArmature::default()
        };

        start_armature.matrices_to_dual_quats();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![dq_to_bone([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])],
        });
        end_actions.insert("Fly".to_string(), Action::new(keyframes));

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }

    // TODO: Function to return these start_actions that we keep using
    #[test]
    fn transpose_actions() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 5.0, 1.0,
            ]))],
        });

        start_actions.insert("Fly".to_string(), Action::new(keyframes));

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            ..BlenderArmature::default()
        };

        start_armature.transpose_actions();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0, 0.0, 0.0, 0.0, 1.0,
            ]))],
        });
        end_actions.insert("Fly".to_string(), Action::new(keyframes));

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }
}
