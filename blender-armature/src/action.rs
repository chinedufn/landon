use std::collections::HashMap;

use crate::Keyframe;

pub use self::action_keyframes::*;

type Frame = u16;

mod action_keyframes;

/// A set of keyframes along with metadata such as pose markers.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct Action {
    keyframes: ActionKeyframes,
    #[serde(default)]
    pose_markers: HashMap<Frame, String>,
}

impl Action {
    #[allow(missing_docs)]
    pub fn new(keyframes: Vec<Keyframe>) -> Self {
        Action {
            pose_markers: HashMap::new(),
            keyframes: ActionKeyframes::new(keyframes),
        }
    }

    /// Each of the key frames for the action.
    pub fn keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes.keyframes()
    }

    /// The smallest keyed frame number in the action.
    pub fn smallest_frame(&self) -> u16 {
        self.keyframes.smallest_frame()
    }

    /// The largest keyed frame number in the action.
    pub fn largest_frame(&self) -> u16 {
        self.keyframes.largest_frame()
    }

    /// The number of frames separating the largest frame from the smallest.
    ///
    /// (largest_frame - smallest_frame)
    ///
    /// So if the largest frame is 30, and the smallest is 20, then the frame duration is 10.
    pub fn frame_duration(&self) -> u16 {
        self.largest_frame() - self.smallest_frame()
    }

    /// Labeled frame times for the action.
    ///
    /// For example, frame 9 might be marked as the "Contact Point".
    pub fn pose_markers(&self) -> &HashMap<Frame, String> {
        &self.pose_markers
    }

    /// See [`Action.method#pose_markers`]
    pub fn pose_markers_mut(&mut self) -> &mut HashMap<Frame, String> {
        &mut self.pose_markers
    }
}

// pub(crate)
impl Action {
    /// We use crate visibility to prevent users from being able to modify keyframes without
    /// updating the cached smallest/largest frame number.
    ///
    /// See [`Action.method#keyframes`]
    pub(crate) fn keyframes_mut(&mut self) -> &mut Vec<Keyframe> {
        self.keyframes.keyframes_mut()
    }
}
