//! Methods and configuration for interpolating keyframed poses, useful for skeletal animation.
//!
//! BlenderArmature currently supports dual quaternion interpolation, but could support 4x4 matrix
//! interpolation if you open an issue/PR.
//!
//! The initial implementation and tests are based off of [skeletal-animation-system](https://github.com/chinedufn/skeletal-animation-system/blob/master/test/skeletal-animation-system.js)
//!
//! A real usage example can be found in the [mesh-visualizer](https://github.com/chinedufn/blender-exporter/tree/master/mesh-visualizer)
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
use std::time::Duration;
use BlenderArmature;
use Bone;

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
    pub should_loop: bool,
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
    pub fn interpolate_bones(&self, opts: InterpolationSettings) -> HashMap<u8, Bone> {
        let mut interpolated_bones = HashMap::new();

        let current_keyframes = self.actions.get(opts.current_action.action_name).unwrap();
        let mut current_keyframe_times: Vec<f32> = current_keyframes
            .iter()
            .map(|(time, _)| time.parse::<f32>().unwrap())
            .collect();
        current_keyframe_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let first_cur_keyframe_time = current_keyframe_times[0];

        let mut current_action_elapsed = opts.current_time + opts.current_action.start_time;
        let mut cur_key_time_to_sample = first_cur_keyframe_time + current_action_elapsed;

        let cur_action_duration = current_keyframe_times.last().unwrap() - first_cur_keyframe_time;

        if current_action_elapsed > cur_action_duration {
            if opts.current_action.should_loop {
                current_action_elapsed = cur_action_duration % cur_action_duration;
            }

            cur_key_time_to_sample = first_cur_keyframe_time + current_action_elapsed;
        }

        // The keyframes surrounding the current key time that we're going to sample
        // TODO: get_surrounding_keyframes()
        let mut cur_action_lower_keyframe = None;
        let mut cur_action_upper_keyframe = None;

        for keyframe_time in current_keyframe_times {
            if cur_key_time_to_sample >= keyframe_time {
                cur_action_lower_keyframe = Some(keyframe_time);
            }
            if cur_key_time_to_sample <= keyframe_time {
                cur_action_upper_keyframe = Some(keyframe_time);
            }

            if cur_action_lower_keyframe.is_some() && cur_action_upper_keyframe.is_some() {
                break;
            }
        }

        let mut cur_action_lower_keyframe = cur_action_lower_keyframe.unwrap();
        let mut cur_action_upper_keyframe = cur_action_upper_keyframe.unwrap();

        let percent_elapsed_into_keyframe = (cur_key_time_to_sample - cur_action_lower_keyframe)
            / (cur_action_upper_keyframe - cur_action_lower_keyframe);

        let blend = 1;

        let lower_keyframe_bones = current_keyframes
            .get(&format!("{}", cur_action_lower_keyframe))
            .unwrap();
        let upper_keyframe_bones = current_keyframes
            .get(&format!("{}", cur_action_upper_keyframe))
            .unwrap();

        for joint_index in opts.joint_indices {
            let lower_bone = &lower_keyframe_bones[joint_index as usize];
            let upper_bone = &upper_keyframe_bones[joint_index as usize];

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
                    .map(|(start, end)| {
                        (end - start) * amount + start
                    })
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

// Tests originally ported from:
//  https://github.com/chinedufn/skeletal-animation-system/tree/8cc52d69f2e4e3f64540a4b6274bcd5fc3c00eee/test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_previous_animation() {
        let mut actions = HashMap::new();
        let mut keyframes = HashMap::new();

        keyframes.insert(
            "0".to_string(),
            vec![Bone::DualQuat(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0])],
        );
        keyframes.insert(
            "2".to_string(),
            vec![Bone::DualQuat(vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0])],
        );
        actions.insert("test".to_string(), keyframes);

        let armature = BlenderArmature {
            actions,
            ..BlenderArmature::default()
        };

        let current_action = ActionSettings {
            action_name: "test",
            start_time: 0.0,
            should_loop: true,
        };

        let interp_settings = InterpolationSettings {
            current_time: 1.5,
            // TODO: armature.get_group_indices(BlenderArmature::BONE_GROUPS_ALL)
            joint_indices: vec![0],
            blend_fn: None,
            current_action,
            previous_action: None,
        };

        let interpolated_bones = armature.interpolate_bones(interp_settings);
        let interpolated_bone = interpolated_bones.get(&0).unwrap();

        let expected_bone = &Bone::DualQuat(vec![0.75, 0.75, 0.75, 0.75, 0.25, 0.25, 0.25, 0.25]);

        assert_eq!(interpolated_bone, expected_bone);
    }
}
