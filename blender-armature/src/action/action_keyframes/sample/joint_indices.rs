/// The joints to sample.
///
/// To only animate, say, the lower body, you'd pass in only the joint indices for the lower
/// body. You'll typically get this vector via:
///   `blender_armature.bone_groups.get('lower_body').unwrap()`
/// assuming that you've created a `lower_body` bone group in Blender.
#[derive(Debug, Clone, Copy)]
pub enum JointIndicesRef<'a> {
    /// Encodes that all of an armature's joints you be used
    All,
    /// Use some subset of the armature's joints.
    ///
    /// Useful for only animating a part of an armature, such as playing a walk animation on the
    /// lower body while the upper body is playing an attack animation.
    Some(&'a [u8]),
}
