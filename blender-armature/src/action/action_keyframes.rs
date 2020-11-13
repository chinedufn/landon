use crate::Keyframe;

mod action_keyframes_serde;

#[derive(Debug, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))]
pub(super) struct ActionKeyframes {
    pub(super) keyframes: Vec<Keyframe>,
    pub(super) smallest_frame: u16,
    pub(super) largest_frame: u16,
}

impl ActionKeyframes {
    /// # Panics
    ///
    /// Panics if the provided list of keyframes is empty.
    pub fn new(keyframes: Vec<Keyframe>) -> Self {
        let mut smallest_frame = u16::max_value();
        let mut largest_frame = u16::min_value();

        for frame in keyframes.iter() {
            smallest_frame = smallest_frame.min(frame.frame);
            largest_frame = largest_frame.max(frame.frame);
        }

        ActionKeyframes {
            keyframes,
            smallest_frame,
            largest_frame,
        }
    }
}
