use std::time::Duration;

/// Settings describing an armature's action.
#[derive(Debug, Clone, Copy)]
pub struct ActionSettings<'a> {
    /// The name of the action (animation) whose keyframes that you want to interpolate
    pub action_name: &'a str,
    /// The time that this action started. By comparing `start_time` to the `current_time`
    /// of your InterpolationSettings we determine how much time has elapsed in the action
    /// and use that to know which keyframes to sample.
    ///
    /// NOTE: Sampling begins from the keyframe time of the first defined frame.
    ///  So if
    ///   - Your first frame is frame 8
    ///   - Your last frame is frame 12
    ///   - Your framerate is 4 frames per second
    ///  Then
    ///   - At t=0s frame 8 will be sampled
    ///   - At t=2s frame 10 will be sampled
    ///
    pub elapsed_time: Duration,
    /// The framerate.
    pub frames_per_second: u8,
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
    pub fn new(
        action_name: &str,
        elapsed_time: Duration,
        frames_per_second: u8,
        should_loop: bool,
    ) -> ActionSettings {
        ActionSettings {
            action_name,
            elapsed_time,
            frames_per_second,
            should_loop,
        }
    }
}
