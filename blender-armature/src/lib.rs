//! Blender files can have armature such as circles, cubes, cylinders, a dragon or any other
//! 3D shape.
//!
//! A armature can be represented as a group of vertices and data about those vertices, such as their
//! normals or UV coordinates.
//!
//! Armaturees can also have metadata, such as the name of it's parent armature (useful for vertex
//! skinning).
//!
//! blender-armature-to-json seeks to be a well tested, well documented exporter for blender armature
//! metadata.
//!
//! You can write data to stdout or to a file. At the onset it will be geared towards @chinedufn's
//! needs - but if you have needs that aren't met feel very free to open an issue.
//!
//! @see https://docs.blender.org/manual/en/dev/modeling/armature/introduction.html - Armature Introduction
//! @see https://github.com/chinedufn/blender-actions-to-json - Exporting blender armatures / actions

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

use crate::serde::serialize_hashmap_deterministic;

pub use self::action::*;
pub use self::bone::*;
pub use self::coordinate_system::*;
pub use self::export::*;
pub use self::interpolate::*;

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
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))]
pub struct BlenderArmature {
    name: String,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    joint_indices: HashMap<String, u8>,
    inverse_bind_poses: Vec<Bone>,
    // TODO: Make private
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
    /// The parent matrices are *not* multiplied in.
    ///
    /// So, if a parent matrix is moved the inverse bind matrix of the child will be the same.
    pub fn inverse_bind_poses(&self) -> &Vec<Bone> {
        &self.inverse_bind_poses
    }

    /// See [`BlenderArmature.inverse_bind_poses()`]
    pub fn inverse_bind_poses_mut(&mut self) -> &mut Vec<Bone> {
        &mut self.inverse_bind_poses
    }

    /// All of the actions defined on the armature, keyed by action name.
    pub fn actions(&self) -> &HashMap<String, Action> {
        &self.actions
    }

    /// See [`BlenderArmature.method#actions`]
    pub fn actions_mut(&mut self) -> &mut HashMap<String, Action> {
        &mut self.actions
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
        for (_name, action) in self.actions.iter_mut() {
            for keyframe in action.keyframes_mut().iter_mut() {
                for (index, bone) in keyframe.bones.iter_mut().enumerate() {
                    *bone = match (&bone, self.inverse_bind_poses[index]) {
                        (Bone::Matrix(bone), Bone::Matrix(inverse_bind)) => {
                            let bone = inverse_bind * (bone.clone());

                            Bone::Matrix(bone)
                        }
                        _ => unimplemented!(),
                    };
                }
            }
        }
    }

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
            Bone::DualQuat(_) => panic!("Cannot transpose dual quat"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::tests::dq_to_bone;

    // TODO: dual_quat_z_up_to_y_up... but we can just get the rendering working first
    // https://github.com/chinedufn/change-mat4-coordinate-system/blob/master/change-mat4-coordinate-system.js
    #[test]
    fn applying_inv_bind_poses() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 6.0, 2.0, 1.0, 7.0, 1.0, 2.0, 5.0, 0.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]))],
        });
        start_actions.insert("Fly".to_string(), Action::new(keyframes));

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            inverse_bind_poses: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 5.0, 1.0,
            ]))],
            ..BlenderArmature::default()
        };

        start_armature.apply_inverse_bind_poses();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame: 1,
            bones: vec![Bone::Matrix(Matrix4::from_column_slice(&[
                1.0, 6.0, 7.0, 1.0, 7.0, 1.0, 27.0, 5.0, 0.0, 4.0, 1.0, 0.0, 0.0, 0.0, 5.0, 1.0,
            ]))],
        });
        end_actions.insert("Fly".to_string(), Action::new(keyframes));

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }

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
