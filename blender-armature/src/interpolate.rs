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

use BlenderArmature;
use std::time::SystemTime;
use Bone;
use std::time::Duration;

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
    pub current_time: SystemTime,
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
    pub previous_action: Option<ActionSettings<'a>>
}

/// Settings for your armature's current action and (optionally) it's previous action.
pub struct ActionSettings<'a> {
    /// The name of the action (animation) whose keyframes that you want to interpolate
    pub action_name: &'a str,
    /// The time that this action started. By comparing `start_time` to the `current_time`
    /// of your InterpolationSettings we determine how much time has elapsed in the action
    /// and use that to know which keyframes to sample.
    pub start_time: Duration,
    /// Whether or not the action should loop if `current_time` - `start_time` is greater than
    /// the duration of the action.
    ///
    /// If you have a 5 second long action with `should_loop: true` then the 7th second would
    /// sample from the 2nd second of the action.
    ///
    /// If `should_loop: false` then 7 seconds in will sample from the 5th second.
    pub should_loop: bool
}

impl BlenderArmature {
    /// Interpolate in between the keyframes of your BlenderArmature. This is useful for
    /// skeletal animation.
    ///
    /// # Panics
    ///
    /// We don't yet support interpolating matrix bones, so we panic if your bones aren't
    /// dual quaternions.
    pub fn interpolate_bones (config: InterpolationSettings) -> Vec<Bone> {
        let interpolated_bones = vec![];

        interpolated_bones
    }
}

// Tests originally ported from:
//  https://github.com/chinedufn/skeletal-animation-system/tree/8cc52d69f2e4e3f64540a4b6274bcd5fc3c00eee/test
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn no_previous_animation() {
        let mut actions = HashMap::new();
        let mut keyframes = HashMap::new();

        keyframes.insert("0", Bone::DualQuat(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]));
        keyframes.insert("2", Bone::DualQuat(vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]));
        actions.insert("test", keyframes);

        let current_action = ActionSettings {
            action_name: "test",
            start_time: Duration::new(0, 0),
            should_loop: true
        };

        let armature = BlenderArmature {
            ..BlenderArmature::default()
        };
    }
}