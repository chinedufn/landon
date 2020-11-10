use std::time::Duration;

pub use self::action_settings::*;

mod action_settings;

/// Settings for how to interpolate your BlenderArmature's bone data. These can be used to do
/// things such as:
///
/// - Interpolate a walk animation to the lower body and a punch animation to the upper body
///   - via `joint_indices`
/// - Interpolate keyframes in slow motion
///   - via slowly increasing the `current_time`
///
/// And more..
#[derive(Debug)]
pub struct InterpolationSettings<'a> {
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
    ///
    /// TODO: Use an enum that has a variant for this slice of indices or a variant for ::ALL
    pub joint_indices: &'a [u8],
    /// Settings for the current action (animation) of this armature.
    pub current_action: &'a ActionSettings<'a>,
    /// Optional settings for the previous action of this armature. This is useful for blending
    /// the last animation that you were playing into the current one.
    pub previous_action: Option<PreviousAction<'a>>,
}

/// Optional settings for the previous action of this armature. This is useful for blending
/// the last animation that you were playing into the current one.
#[derive(Debug)]
pub struct PreviousAction<'a> {
    /// The action that we are blending away fro,
    pub action: &'a ActionSettings<'a>,
    /// 0.0 means to source from your previous animation, 1.0 your current animation, and anything
    /// in between controls how much of your previous animation to use vs. your next.
    ///
    /// This is used to control how quickly your previous_action blends into your current_action.
    ///
    /// If you supply a previous_animation your previous_action will be blended into your
    /// current_action using your create_interp_param.
    ///
    /// ex:
    /// ```
    /// use std::time::Duration;
    ///
    /// // Blend previous_action into current_action linearly over 5 seconds
    /// let create_interp_param_creator = |elapsed_time: Duration| 0.2 * elapsed_time.as_secs_f32();
    /// ```
    pub create_interp_param: fn(Duration) -> f32,
}
