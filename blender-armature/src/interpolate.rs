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
//! let armature = BlenderArmature::from_json(r#"...");
//! let config = InterpConfig {
//!   // TODO ...
//! };
//! let bones = armature.interpolate_bones(config);
//! // ...
//! // ... Pass your bone data to your vertex shader ...
//! // ...
//! ```

use std::collections::HashMap;
use crate::BlenderArmature;
use crate::Bone;
use crate::Keyframe;

/// Settings for how to interpolate your BlenderArmature's bone data. These can be used to do
/// things such as:
///
/// - Interpolate a walk animation to the lower body and a punch animation to the upper body
///   - via `joint_indices`
/// - Interpolate keyframes in slow motion
///   - via slowly increasing the `current_time`
///
/// And more..
pub struct InterpolationSettings<'a> {
    /// The current time will get compared to the start time of your current / previous animations.
    /// Bones will be interpolated based on the seconds elapsed.
    ///  (current_time - {current_animation,start_animation}.start_time)
    pub current_time: f32,
    /// The joints that you want to interpolate. To interpolate the first, third and fourth bone
    /// you'd set this to vec![0, 2, 3].
    ///
    /// To animate an entire armature you could pass in `vec![0, 1, .., n - 1]` where `n` is the
    /// number of bones in the armature. Usually via:
    ///   `blender_armature.bone_groups.get(BlenderArmature::BONE_GROUP_ALL).unwrap()`
    ///
    /// To only animate, say, the lower body, you'd pass in only the joint indices for the lower
    /// body. You'll typically get this vector via:
    ///   `blender_armature.bone_groups.get('lower_body').unwrap()`
    /// assuming that you've created a `lower_body` bone group in Blender.
    pub joint_indices: Vec<u8>,
    /// Your blend_fn returns a number between `0.0` and `1.0`. This is used to control how
    /// quickly your previous_action blends into your current_action.
    ///
    /// By default, of no `blend_fn` is specified, your previous_action will blend into your
    /// current_action linearly over 0.2s
    ///
    /// 0.0 means to source from your previous animation, 1.0 your current animation, and anything
    /// in between controls how much of your previous animation to use vs. your next.
    ///
    /// If you supply a previous_animation your previous_action will be blended into your
    /// current_action using your blend_fn.
    /// ex:
    /// ```
    /// // Blend previous_action into current_action linearly over 5 seconds
    /// let blend_fn = |delta_seconds: f32| 0.2 * delta_seconds;
    /// ```
    pub blend_fn: Option<fn(f32) -> f32>,
    /// Settings for the current action (animation) of this armature.
    pub current_action: ActionSettings<'a>,
    /// Optional settings for the previous action of this armature. This is useful for blending
    /// the last animation that you were playing into the current one.
    pub previous_action: Option<ActionSettings<'a>>,
}

/// Settings for your armature's current action and (optionally) it's previous action.
pub struct ActionSettings<'a> {
    /// The name of the action (animation) whose keyframes that you want to interpolate
    pub action_name: &'a str,
    /// The time that this action started. By comparing `start_time` to the `current_time`
    /// of your InterpolationSettings we determine how much time has elapsed in the action
    /// and use that to know which keyframes to sample.
    /// Note that when sampling your animation we start from the keyframe time of the first
    /// keyfame, not from time 0.0.
    /// So if you have keyframes 1.0, 3.0, and 6.0 seconds and your elapsed time is
    /// 1.5 seconds we sample at time 2.5s, not time 1.5
    pub start_time: f32,
    /// Whether or not the action should loop if `current_time` - `start_time` is greater than
    /// the duration of the action.
    ///
    /// If you have a 5 second long action with `should_loop: true` then the 7th second would
    /// sample from the 2nd second of the action.
    ///
    /// If `should_loop: false` then 7 seconds in will sample from the 5th second.
    ///
    /// `true` is for repeating actions such as walk cycles, `false` might be used for a one off
    /// punch animation that shouldn't repeat.
    pub should_loop: bool,
}

impl<'a> ActionSettings<'a> {
    /// Creates new action settings for a specified action name
    pub fn new(action_name: &str, start_time: f32, should_loop: bool) -> ActionSettings {
        ActionSettings {
            action_name,
            start_time,
            should_loop,
        }
    }
}

impl BlenderArmature {
    /// Interpolate in between the keyframes of your BlenderArmature. This is useful for
    /// skeletal animation.
    ///
    /// We return a hashmap so that you can easily merge the results of interpolating
    /// different sets of bone groups.
    ///
    /// # Panics
    ///
    /// We don't yet support interpolating matrix bones, so we panic if your bones aren't
    /// dual quaternions.
    ///
    /// Panics if you pass in previous actions that do not have the exact same joint indices
    /// as your current action.
    ///
    /// # TODO
    ///
    /// - [ ] Return Result<HashMap<u8, Bone>, InterpolationError>
    /// - [ ] error if clock time is negative
    pub fn interpolate_bones(&self, opts: &InterpolationSettings) -> HashMap<u8, Bone> {
        let mut interpolated_bones = self.interpolate_action(&opts, &opts.current_action);

        if let Some(ref previous_action) = opts.previous_action {
            let previous_bones = self.interpolate_action(&opts, &previous_action);

            let cur_anim_elapsed_time = opts.current_time - opts.current_action.start_time;

            let blend_func = if let Some(blend_func) = opts.blend_fn {
                blend_func
            } else {
                |dt_seconds: f32| (2.0 as f32 * dt_seconds).min(1.0)
            };

            interpolated_bones = interpolated_bones
                .iter()
                .zip(previous_bones.iter())
                .map(
                    |((cur_joint_idx, cur_action_bone), (prev_joint_idx, prev_action_bone))| {
                        if prev_joint_idx != cur_joint_idx {
                            panic!("We do not currently support the current action having different joints than the previous action");
                        }

                        let mut prev_action_bone = prev_action_bone.vec();
                        let cur_action_bone = cur_action_bone.vec();

                        // Get the dot product of the start and end rotation quaternions. If the
                        // dot product is negative we negative the rotation portion of the first
                        // dual quaternion in order to ensure the shortest path rotation.
                        // http://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
                        if dot_product(&prev_action_bone, &cur_action_bone) < 0.0 {
                            prev_action_bone[0] = -prev_action_bone[0];
                            prev_action_bone[1] = -prev_action_bone[1];
                            prev_action_bone[2] = -prev_action_bone[2];
                            prev_action_bone[3] = -prev_action_bone[3];
                        }

                        let new_bone: Vec<f32> = prev_action_bone
                            .iter()
                            .zip(cur_action_bone.iter())
                            .map(|(prev, cur)| {
                                prev + (cur - prev) * blend_func(cur_anim_elapsed_time)
                            })
                            .collect();

                        (*cur_joint_idx, Bone::DualQuat(new_bone))
                    },
                )
                .collect()
        };

        interpolated_bones
    }

    fn interpolate_action(
        &self,
        opts: &InterpolationSettings,
        action: &ActionSettings,
    ) -> HashMap<u8, Bone> {
        let mut interpolated_bones = HashMap::new();

        let keyframes = self.actions.get(action.action_name).unwrap();

        let first_keyframe = &keyframes[0];

        let mut time_elapsed_since_first_keyframe = opts.current_time - action.start_time;
        let mut key_time_to_sample =
            first_keyframe.frame_time_secs + time_elapsed_since_first_keyframe;

        let action_duration =
            keyframes.last().unwrap().frame_time_secs - first_keyframe.frame_time_secs;

        if time_elapsed_since_first_keyframe > action_duration {
            if action.should_loop {
                time_elapsed_since_first_keyframe =
                    time_elapsed_since_first_keyframe % action_duration;
            } else {
                time_elapsed_since_first_keyframe = action_duration;
            }

            key_time_to_sample = first_keyframe.frame_time_secs + time_elapsed_since_first_keyframe;
        }

        let (action_lower_keyframe, action_upper_keyframe) =
            get_surrounding_keyframes(keyframes, key_time_to_sample);

        let percent_elapsed_into_keyframe = if action_lower_keyframe == action_upper_keyframe {
            0.0
        } else {
            (key_time_to_sample - action_lower_keyframe.frame_time_secs)
                / (action_upper_keyframe.frame_time_secs - action_lower_keyframe.frame_time_secs)
        };

        for joint_index in &opts.joint_indices {
            let joint_index = *joint_index;

            let lower_bone = &action_lower_keyframe.bones[joint_index as usize];
            let upper_bone = &action_upper_keyframe.bones[joint_index as usize];

            let interpolated_bone =
                interpolate_bones(&lower_bone, &upper_bone, percent_elapsed_into_keyframe);
            interpolated_bones.insert(joint_index, interpolated_bone);
        }

        interpolated_bones
    }
}

fn interpolate_bones(start_bone: &Bone, end_bone: &Bone, amount: f32) -> Bone {
    match start_bone {
        &Bone::DualQuat(ref start_dual_quat) => match end_bone {
            &Bone::DualQuat(ref end_dual_quat) => {
                let interpolated_dual_quat: Vec<f32> = start_dual_quat
                    .iter()
                    .zip(end_dual_quat.iter())
                    .map(|(start, end)| (end - start) * amount + start)
                    .collect();

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

// If you're sampling time 1.5seconds and there are three keyframes, 0.0s, 1.8s, 2.2s the
// surrounding keyframes are 0.0s and 1.8s
fn get_surrounding_keyframes(
    keyframes: &Vec<Keyframe>,
    key_time_to_sample: f32,
) -> (&Keyframe, &Keyframe) {
    let mut action_lower_keyframe = None;
    let mut action_upper_keyframe = None;

    'lower_upper: for keyframe in keyframes {
        if key_time_to_sample >= keyframe.frame_time_secs {
            action_lower_keyframe = Some(keyframe);
        }
        if key_time_to_sample <= keyframe.frame_time_secs {
            action_upper_keyframe = Some(keyframe);
        }

        if action_lower_keyframe.is_some() && action_upper_keyframe.is_some() {
            break 'lower_upper;
        }
    }

    (
        action_lower_keyframe.unwrap(),
        action_upper_keyframe.unwrap(),
    )
}

fn dot_product(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

// Tests originally ported from:
//  https://github.com/chinedufn/skeletal-animation-system/tree/8cc52d69f2e4e3f64540a4b6274bcd5fc3c00eee/test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keyframe;

    // TODO: Breadcrumb - make out first case create a test case then run it through some generic
    // test function that we can re-use
    struct DualQuatTestCase<'a> {
        description: String,
        keyframes: Vec<TestKeyframe>,
        expected_bone: Vec<f32>,
        interp_settings: InterpolationSettings<'a>,
    }

    struct TestKeyframe {
        frame: f32,
        bone: Vec<f32>,
    }

    #[test]
    fn no_previous_animation() {
        DualQuatTestCase {
            description: "".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 0.0,
                    bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframe {
                    frame: 2.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: vec![0.75, 0.75, 0.75, 0.75, 0.25, 0.25, 0.25, 0.25],
            interp_settings: InterpolationSettings {
                current_time: 1.5,
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                joint_indices: vec![0],
                blend_fn: None,
                current_action: ActionSettings::new("test", 0.0, true),
                previous_action: None,
            },
        };
    }

    #[test]
    fn looping_action() {
        DualQuatTestCase {
            description: "Verify that the action gets looped by choosing a current_time > duration".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 1.0,
                    bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframe {
                    frame: 3.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            interp_settings: InterpolationSettings {
                current_time: 4.0,
                joint_indices: vec![0],
                blend_fn: None,
                current_action: ActionSettings::new("test", 0.0, true),
                previous_action: None,
            },
        };
    }

    #[test]
    fn non_looping_animation() {
        DualQuatTestCase {
            description: "If you are not looping we should sample from the final frame if exceeded".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 3.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframe {
                    frame: 5.0,
                    bone: vec![3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
                },
            ],
            expected_bone: vec![3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
            interp_settings: InterpolationSettings {
                current_time: 7.0,
                joint_indices: vec![0],
                blend_fn: None,
                current_action: ActionSettings::new("test", 0.0, false),
                previous_action: None,
            },
        };
    }

    #[test]
    fn previous_animation_does_not_loop() {
        DualQuatTestCase {
            description: "Make sure should_loop: false works for previous animation".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 1.0,
                    bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframe {
                    frame: 3.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframe {
                    frame: 5.0,
                    bone: vec![3.0, 3.0, 3.0, 3.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframe {
                    frame: 7.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            expected_bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            interp_settings: InterpolationSettings {
                current_time: 10.0,
                joint_indices: vec![0],
                blend_fn: None,
                current_action: ActionSettings::new("test", 10.0, true),
                previous_action: Some(ActionSettings::new("test", 0.0, false)),
            },
        };
    }

    #[test]
    fn blend_out_previous_action() {
        DualQuatTestCase {
            description: "Previous action gets blended into the new current action".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 0.0,
                    bone: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                },
                TestKeyframe {
                    frame: 3.0,
                    bone: vec![3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
                },
                TestKeyframe {
                    frame: 5.0,
                    bone: vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
                },
                TestKeyframe {
                    frame: 8.0,
                    bone: vec![8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                },
            ],
            expected_bone: vec![3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
            interp_settings: InterpolationSettings {
                current_time: 10.0,
                joint_indices: vec![0],
                blend_fn: Some(two_second_blend_func),
                current_action: ActionSettings::new("test", 9.0, true),
                previous_action: Some(ActionSettings::new("test", 5.0, false)),
            },
        };
    }

    #[test]
    fn trimmed_down_armature_that_was_panicking_when_calling_interpolate() {
        // Ripped this out of the skinned_letter_f.blend's JSON
        let armature = r#"
        {
          "actions": {
            "Twist": [
              {"frame_time_secs": 0.0, "bones": [{"Matrix": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]}]},
              {"frame_time_secs": 2.5, "bones": [{"Matrix": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]}]},
              {"frame_time_secs": 4.166667, "bones":  [{"Matrix": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]}]}
            ]
          },
          "inverse_bind_poses": [],
          "joint_index": {}
        }
    "#;
        let mut armature = BlenderArmature::from_json(armature).unwrap();
        armature.actions_to_dual_quats();

        let interp_opts = InterpolationSettings {
            current_time: 209.109,
            // TODO: self.get_bone_group(BlenderArmature::ALL_BONES)
            joint_indices: vec![0],
            blend_fn: None,
            current_action: ActionSettings::new("Twist", 0.0, true),
            previous_action: None,
        };
        // Just making sure that this no longer panics..
        armature.interpolate_bones(&interp_opts);
    }

    #[test]
    fn current_time_equals_start_time () {
        DualQuatTestCase {
            description: "Ensure that current_time == start_time works".to_string(),
            keyframes: vec![
                TestKeyframe {
                    frame: 0.0,
                    // This will be the expected bone since we're 0 seconds into our animation
                    bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                },
                TestKeyframe {
                    frame: 2.0,
                    bone: vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                },
            ],
            // Same as the first bone in the animation
            expected_bone: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            interp_settings: InterpolationSettings {
                current_time: 0.0,
                // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
                joint_indices: vec![0],
                blend_fn: None,
                current_action: ActionSettings::new("test", 0.0, true),
                previous_action: None,
            },
        };
    }

    impl<'a> Drop for DualQuatTestCase<'a> {
        fn drop(&mut self) {
            let mut actions = HashMap::new();
            let mut keyframes = vec![];

            for keyframe in self.keyframes.iter() {
                keyframes.push(Keyframe {
                    frame_time_secs: keyframe.frame,
                    bones: vec![Bone::DualQuat(keyframe.bone.clone())],
                });
            }

            actions.insert("test".to_string(), keyframes);

            let armature = BlenderArmature {
                actions,
                ..BlenderArmature::default()
            };

            let interpolated_bones = armature.interpolate_bones(&self.interp_settings);
            let interpolated_bone = interpolated_bones.get(&0).unwrap();

            assert_eq!(interpolated_bone.vec(), self.expected_bone, "{}", self.description);
        }
    }

    fn two_second_blend_func(dt_seconds: f32) -> f32 {
        (0.5 as f32 * dt_seconds).min(1.0)
    }
}
