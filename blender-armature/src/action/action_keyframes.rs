use crate::Keyframe;

pub use self::sample::*;
use std::ops::Deref;

mod action_keyframes_serde;
mod sample;

/// All of the keyframes in an action.
#[derive(Debug, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))]
pub struct ActionKeyframes {
    keyframes: Vec<Keyframe>,
    smallest_frame: u16,
    largest_frame: u16,
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

        let mut keyframes = ActionKeyframes {
            keyframes,
            smallest_frame,
            largest_frame,
        };

        keyframes.sort_keyframes_ascending();

        keyframes
    }

    /// Keyframes are guaranteed to be ordered from smallest frame first to largest frame last.
    pub fn keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes
    }

    pub fn smallest_frame(&self) -> u16 {
        self.smallest_frame
    }

    pub fn largest_frame(&self) -> u16 {
        self.largest_frame
    }

    /// Insert a keyframe into the list of keyframes for the action
    pub fn insert_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        self.sort_keyframes_ascending();
    }

    fn sort_keyframes_ascending(&mut self) {
        self.keyframes.sort_by(|a, b| a.frame.cmp(&b.frame));
    }
}

impl ActionKeyframes {
    /// We use crate visibility to prevent users from being able to modify keyframes without
    /// updating the cached smallest/largest frame number.
    pub(crate) fn keyframes_mut(&mut self) -> &mut Vec<Keyframe> {
        &mut self.keyframes
    }
}

impl Deref for ActionKeyframes {
    type Target = Vec<Keyframe>;

    fn deref(&self) -> &Self::Target {
        &self.keyframes
    }
}
