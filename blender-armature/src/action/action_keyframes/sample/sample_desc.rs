use std::time::Duration;

/// Describes how to sample animation keyframes
#[derive(Debug, Clone, Copy)]
pub struct SampleDesc<'a> {
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
    /// NOTE: Sampling begins from the keyframe time of the first defined frame.
    ///  So if
    ///   - Your first frame is frame 8
    ///   - Your last frame is frame 12
    ///   - Your framerate is 4 frames per second
    ///  Then
    ///   - At t=0s frame 8 will be sampled
    ///   - At t=2s frame 10 will be sampled
    pub frame_offset: FrameOffset,
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

/// Describes some number of frames to offset from some other frame.
/// Useful for sampling keyframes.
#[derive(Debug, Clone, Copy)]
pub struct FrameOffset(f32);

impl FrameOffset {
    #[allow(missing_docs)]
    pub fn new(frame_offset: f32) -> Self {
        FrameOffset(frame_offset)
    }

    /// Calculate a frame offset based on the amount of time elapsed and the framerate
    pub fn new_with_elapsed_time_and_frames_per_second(
        elapsed_time: Duration,
        frames_per_second: u8,
    ) -> Self {
        Self(frames_per_second as f32 * elapsed_time.as_secs_f32())
    }

    /// Return the inner float representing the frame offset.
    pub fn get(&self) -> f32 {
        self.0
    }
}
