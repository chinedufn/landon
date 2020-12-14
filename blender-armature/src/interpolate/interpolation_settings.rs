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
    pub action: ActionSettings<'a>,
}
