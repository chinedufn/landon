pub use self::frame_offset::*;

mod frame_offset;

/// Describes how to sample animation keyframes
#[derive(Debug, Clone, Copy)]
pub struct SampleDesc {
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
