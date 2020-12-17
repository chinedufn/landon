use std::time::Duration;

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
