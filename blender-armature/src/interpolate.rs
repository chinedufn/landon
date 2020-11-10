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

use std::collections::BTreeMap;
use std::time::Duration;

use crate::BlenderArmature;
use crate::Bone;

pub use self::interpolation_settings::*;
use std::ops::Deref;

mod interpolate_action;
mod interpolation_settings;

/// Returns 0.0 if no time has elapsed.
/// Returns 0.5 if 100 milliseconds have elapsed.
/// Returns 1.0 if >= 200 milliseconds have elapsed
pub fn linear_200_milliseconds(elapsed: Duration) -> f32 {
    (5.0 * elapsed.as_secs_f32()).min(1.0)
}

/// Bones that were interpolated based on an armature's action.
#[derive(Debug)]
pub struct InterpolatedBones {
    bones: BTreeMap<u8, Bone>,
}

impl Deref for InterpolatedBones {
    type Target = BTreeMap<u8, Bone>;

    fn deref(&self) -> &Self::Target {
        &self.bones
    }
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
    pub fn interpolate_bones(&self, opts: InterpolationSettings) -> InterpolatedBones {
        let current_action_bones =
            self.interpolate_action(&opts.current_action, opts.joint_indices);

        let bones = match opts.previous_action {
            None => current_action_bones,
            Some(previous_action) => {
                let previous_action_bones =
                    self.interpolate_action(&previous_action.action, opts.joint_indices);

                interpolate_bones(
                    &previous_action_bones,
                    &current_action_bones,
                    (previous_action.create_interp_param)(opts.current_action.elapsed_time),
                )
            }
        };

        InterpolatedBones { bones }
    }
}

fn interpolate_bones(
    start: &BTreeMap<u8, Bone>,
    end: &BTreeMap<u8, Bone>,
    interp_param: f32,
) -> BTreeMap<u8, Bone> {
    start
        .iter()
        .zip(end.iter())
        .map(
            |((prev_joint_idx, prev_action_bone), (cur_joint_idx, cur_action_bone))| {
                // TODO: We were using a hashmap where the iteration order isn't guaranteed and hence we would hit this condition.
                // Really just need to refactor all of landon now that we're much more experienced with Rust.
                if prev_joint_idx != cur_joint_idx {
                    panic!("We do not currently support the current action having different joints than the previous action");
                }

                // FIXME: Ditch clones
                let prev = prev_action_bone.as_slice();
                let mut prev_action_bone: [f32; 8] = [0.0; 8];
                prev_action_bone.copy_from_slice(prev);

                // Get the dot product of the start and end rotation quaternions. If the
                // dot product is negative we negate the first dual quaternion in order to
                // ensure the shortest path rotation.
                //
                // http://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
                // https://github.com/chinedufn/skeletal-animation-system/blob/9ae17c5b23759f7147bf7c464564e32a09e619ef/src/blend-dual-quaternions.js#L59
                if dot_product(&prev_action_bone, cur_action_bone.as_slice()) < 0.0 {
                    prev_action_bone[0] = -prev_action_bone[0];
                    prev_action_bone[1] = -prev_action_bone[1];
                    prev_action_bone[2] = -prev_action_bone[2];
                    prev_action_bone[3] = -prev_action_bone[3];
                    prev_action_bone[4] = -prev_action_bone[4];
                    prev_action_bone[5] = -prev_action_bone[5];
                    prev_action_bone[6] = -prev_action_bone[6];
                    prev_action_bone[7] = -prev_action_bone[7];
                }

                let _new_bone = [0.0; 8];

                let new_bone = interpolate_bone(&Bone::DualQuat(prev_action_bone), &cur_action_bone, interp_param);

                (*cur_joint_idx, new_bone)
            },
        )
        .collect()
}

fn interpolate_bone(start_bone: &Bone, end_bone: &Bone, amount: f32) -> Bone {
    match start_bone {
        &Bone::DualQuat(ref start_dual_quat) => match end_bone {
            &Bone::DualQuat(ref end_dual_quat) => {
                let mut interpolated_dual_quat: [f32; 8] = [0.0; 8];

                for index in 0..8 {
                    let start = start_dual_quat[index];
                    let end = end_dual_quat[index];
                    interpolated_dual_quat[index] = (end - start) * amount + start;
                }

                Bone::DualQuat(interpolated_dual_quat)
            }
            _ => panic!(
                "You may only interpolate bones of the same type. Please convert\
                 your end bone into a dual quaternion before interpolating"
            ),
        },
        &Bone::Matrix(ref _matrix) => unimplemented!(),
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

// Tests originally ported from:
//  https://github.com/chinedufn/skeletal-animation-system/tree/8cc52d69f2e4e3f64540a4b6274bcd5fc3c00eee/test
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Action, Keyframe};

    use super::*;

    struct DualQuatTestCase<'a> {
        keyframes: Vec<TestKeyframeDualQuat>,
        expected_bone: [f32; 8],
        interp_settings: InterpolationSettings<'a>,
    }

    struct TestKeyframeDualQuat {
        frame: u16,
        bone: [f32; 8],
    }

    const ONE_FPS: u8 = 1;

    /// Verify that if there is no previous animation only the current animation is intrepolated.
    #[test]
    fn no_previous_animation() {
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
            interp_settings: InterpolationSettings {
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new(
                    "test",
                    Duration::from_secs_f32(1.5),
                    ONE_FPS,
                    true,
                ),
                previous_action: None,
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
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new("test", Duration::from_secs(4), ONE_FPS, true),
                previous_action: None,
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
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new(
                    "test",
                    Duration::from_secs_f32(2.5),
                    ONE_FPS,
                    true,
                ),
                previous_action: None,
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
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new(
                    "test",
                    Duration::from_secs(7),
                    ONE_FPS,
                    false,
                ),
                previous_action: None,
            },
        }
        .test();
    }

    /// Verify that the previous action and current action are blended together based on the
    /// provided blend function.
    #[test]
    fn blend_out_previous_action() {
        DualQuatTestCase {
            keyframes: vec![
                TestKeyframeDualQuat {
                    frame: 0,
                    bone: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframeDualQuat {
                    frame: 3,
                    bone: [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
                },
                TestKeyframeDualQuat {
                    frame: 5,
                    bone: [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
                },
                TestKeyframeDualQuat {
                    frame: 8,
                    bone: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                },
            ],
            expected_bone: [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new("test", Duration::from_secs(1), ONE_FPS, true),
                previous_action: Some(PreviousAction {
                    action: &ActionSettings::new("test", Duration::from_secs(5), ONE_FPS, false),
                    create_interp_param: two_second_blend_func,
                }),
            },
        }
        .test();
    }

    /// Verify that if the previous animation is set to not loop and enough time has elapsed for the
    /// previous animation to be passed the end frame that we sample the end frame when blending
    /// in the previous animation.
    #[test]
    fn non_looping_previous_animation() {
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
                TestKeyframeDualQuat {
                    frame: 5,
                    bone: [3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframeDualQuat {
                    frame: 7,
                    bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new("test", Duration::from_secs(0), ONE_FPS, true),
                previous_action: Some(PreviousAction {
                    action: &ActionSettings::new("test", Duration::from_secs(10), ONE_FPS, false),
                    create_interp_param: linear_200_milliseconds,
                }),
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
            interp_settings: InterpolationSettings {
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new("test", Duration::from_secs(0), ONE_FPS, true),
                previous_action: None,
            },
        }
        .test();
    }

    /// Verify that the frames per second are factored in when sampling the current and previous
    /// action.
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
            expected_bone: [6., 6., 6., 6., 6., 6., 6., 6.],
            interp_settings: InterpolationSettings {
                joint_indices: &vec![0][..],
                current_action: &ActionSettings::new(
                    "test",
                    Duration::from_secs_f32(0.1),
                    10,
                    false,
                ),
                previous_action: Some(PreviousAction {
                    action: &ActionSettings::new(
                        "test",
                        Duration::from_secs_f32(0.002),
                        100,
                        false,
                    ),
                    create_interp_param: linear_200_milliseconds,
                }),
            },
        }
        .test();
    }

    impl<'a> DualQuatTestCase<'a> {
        fn test(self) {
            let mut actions = HashMap::new();
            let mut keyframes = vec![];

            for keyframe in self.keyframes.iter() {
                keyframes.push(Keyframe {
                    frame: keyframe.frame,
                    bones: vec![Bone::DualQuat(keyframe.bone.clone())],
                });
            }

            actions.insert("test".to_string(), Action::new(keyframes));

            let armature = BlenderArmature {
                actions,
                ..BlenderArmature::default()
            };

            let interpolated_bones = armature.interpolate_bones(self.interp_settings);
            let interpolated_bone = interpolated_bones.get(&0).unwrap();

            assert_eq!(interpolated_bone.as_slice(), &self.expected_bone,);
        }
    }

    fn two_second_blend_func(elapsed: Duration) -> f32 {
        (0.5 * elapsed.as_secs_f32()).min(1.0)
    }
}
