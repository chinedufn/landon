//! Methods and configuration for interpolating keyframed poses, useful for skeletal animation.
//!
//! BlenderArmature currently supports dual quaternion interpolation, but could support 4x4 matrix
//! interpolation if you open an issue/PR.
//!
//! The initial implementation and tests are based off of [skeletal-animation-system](https://github.com/chinedufn/skeletal-animation-system/blob/master/test/skeletal-animation-system.js)
//!
//! A real usage example can be found in the [mesh-visualizer](https://github.com/chinedufn/landon/tree/master/mesh-visualizer)
//!
//! # Examples
//!
//! ```ignore
//! extern crate blender_armature;
//!
//! use blender_armature::InterpolationConfig as InterpConf;
//!
//! // TODO: Tell reader how to get some JSON
//! let armature = BlenderArmature::from_json(r#"..."#);
//! let config = InterpConfig {
//!   // TODO ...
//! };
//! let bones = armature.interpolate_bones(config);
//! // ...
//! // ... Pass your bone data to your vertex shader ...
//! // ...
//! ```

use std::time::Duration;

use crate::{BlenderArmature, Bone, JointIndicesRef, SampleDesc};

pub use self::interpolated_bones::*;
use std::collections::BTreeMap;

mod interpolated_bones;
mod sample_action;

/// Returns 0.0 if no time has elapsed.
/// Returns 0.5 if 100 milliseconds have elapsed.
/// Returns 1.0 if >= 200 milliseconds have elapsed
pub fn linear_200_milliseconds(elapsed: Duration) -> f32 {
    (5.0 * elapsed.as_secs_f32()).min(1.0)
}

impl BlenderArmature {
    /// Interpolate in between the keyframes of your BlenderArmature. This is useful for
    /// skeletal animation.
    ///
    /// We return a map so that you can easily merge the the interpolating bones with other
    /// interpolations. This is useful when you are combining multiple bone groups.
    ///
    /// # Panics
    ///
    /// We don't currently interpolating matrix bones, so we panic if your bones aren't
    /// dual quaternions.
    ///
    /// Panics if you pass in previous actions that do not have the exact same joint indices
    /// as your current action.
    ///
    /// # TODO
    ///
    /// - [ ] Return Result<HashMap<u8, Bone>, InterpolationError>
    /// - [ ] error if clock time is negative
    pub fn interpolate_bones(
        &self,
        action_name: &str,
        joint_indices: JointIndicesRef,
        sample_desc: SampleDesc,
    ) -> BTreeMap<u8, Bone> {
        self.sample_action(action_name, joint_indices, sample_desc)
    }
}

// Tests originally ported from:
//  https://github.com/chinedufn/skeletal-animation-system/tree/8cc52d69f2e4e3f64540a4b6274bcd5fc3c00eee/test
#[cfg(test)]
pub(super) mod tests {

    use crate::{Bone, BoneKeyframe, FrameOffset, JointIndicesRef, Keyframe, SampleDesc};

    use super::*;
    use crate::test_util::{action_name, action_with_keyframes, BONE_IDX};
    use nalgebra::{DualQuaternion, Quaternion};

    struct DualQuatTestCase {
        keyframes: Vec<TestKeyframeDualQuat>,
        expected_bone: [f32; 8],
        sample_desc: SampleDesc,
    }

    struct TestKeyframeDualQuat {
        frame: u16,
        bone: [f32; 8],
    }

    const ONE_FPS: u8 = 1;

    /// Verify that we blend properly when the elapsed time has not yet exceeded the animation's
    /// duration.
    #[test]
    fn less_than_total_duration() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 0,
                    bone: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframeDualQuat {
                    frame: 2,
                    bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: [0.75, 0.75, 0.75, 0.75, 0.25, 0.25, 0.25, 0.25],
            sample_desc: SampleDesc {
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs_f32(1.5),
                    ONE_FPS,
                ),
                should_loop: true,
            },
        }
        .test();
    }

    /// Verify that the amount of time elapsed is larger than the total duration of the animation
    /// and looping is enabled we loop around from the beginning when we sample.
    #[test]
    fn looping_action() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 1,
                    bone: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframeDualQuat {
                    frame: 3,
                    bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            sample_desc: SampleDesc {
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs(4),
                    ONE_FPS,
                ),
                should_loop: true,
            },
        }
        .test();
    }

    /// Verify that looping works properly when the keyframes are not provided in ascending order.
    #[test]
    fn looping_order_bugfix() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 1,
                    bone: [8.0, 8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframeDualQuat {
                    frame: 2,
                    bone: [20.0, 20.0, 20.0, 20.0, 00.0, 00.0, 0.0, 0.0],
                },
                TestKeyframeDualQuat {
                    frame: 0,
                    bone: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: [4.0, 4.0, 4.0, 4.0, 0.0, 0.0, 0.0, 0.0],
            sample_desc: SampleDesc {
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs_f32(2.5),
                    ONE_FPS,
                ),
                should_loop: true,
            },
        }
        .test();
    }

    /// Verify that if the elapsed time exceeds the duration of the animation and looping is
    /// disabled then we sample the last keyframe.
    #[test]
    fn non_looping_animation() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 3,
                    bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframeDualQuat {
                    frame: 5,
                    bone: [3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
                },
            ],
            expected_bone: [3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
            sample_desc: SampleDesc {
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs(7),
                    ONE_FPS,
                ),
                should_loop: false,
            },
        }
        .test();
    }

    /// Verify that if no time has elapsed we sample from the first frame in the action.
    #[test]
    fn no_elapsed_time() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 0,
                    // This will be the expected bone since we're 0 seconds into our animation
                    bone: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframeDualQuat {
                    frame: 2,
                    bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            // Same as the first bone in the animation
            expected_bone: [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            sample_desc: SampleDesc {
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs(0),
                    ONE_FPS,
                ),
                should_loop: true,
            },
        }
        .test();
    }

    /// Verify that the frames per second are factored in when sampling the action.
    #[test]
    fn uses_frames_per_second() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 0,
                    bone: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframeDualQuat {
                    frame: 10,
                    bone: [100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0],
                },
            ],
            expected_bone: [20., 20., 20., 20., 20., 20., 20., 20.],
            sample_desc: SampleDesc {
                frame_offset: FrameOffset::new_with_elapsed_time_and_frames_per_second(
                    Duration::from_secs_f32(0.2),
                    10,
                ),
                should_loop: false,
            },
        }
        .test();
    }

    impl DualQuatTestCase {
        fn test(self) {
            let mut keyframes = vec![];

            for keyframe in self.keyframes.iter() {
                keyframes.push(BoneKeyframe::new(keyframe.frame, dq_to_bone(keyframe.bone)));
            }

            let armature = BlenderArmature {
                bone_space_actions: action_with_keyframes(keyframes),
                ..BlenderArmature::default()
            };

            let interpolated_bones = armature.interpolate_bones(
                &action_name(),
                JointIndicesRef::Some(&[BONE_IDX]),
                self.sample_desc,
            );
            let interpolated_bone = interpolated_bones.get(&BONE_IDX).unwrap();

            assert_eq!(interpolated_bone, &dq_to_bone(self.expected_bone));
        }
    }

    pub(crate) fn dq_to_bone(dq: [f32; 8]) -> Bone {
        Bone::DualQuat(DualQuaternion::from_real_and_dual(
            Quaternion::new(dq[0], dq[1], dq[2], dq[3]),
            Quaternion::new(dq[4], dq[5], dq[6], dq[7]),
        ))
    }
}
