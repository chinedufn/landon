use crate::Keyframe;
use std::collections::HashMap;

type Frame = u16;

/// A set of keyframes along with metadata such as pose markers.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))]
pub struct Action {
    keyframes: Vec<Keyframe>,
    pose_markers: HashMap<Frame, String>,
}

impl Action {
    #[allow(missing_docs)]
    pub fn new(keyframes: Vec<Keyframe>) -> Self {
        Action {
            keyframes,
            pose_markers: HashMap::new(),
        }
    }

    /// Each of the key frames for the action.
    pub fn keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes
    }

    /// See [`Action.method#keyframes`]
    pub fn keyframes_mut(&mut self) -> &mut Vec<Keyframe> {
        &mut self.keyframes
    }

    /// Labeled frame times for the action.
    ///
    /// For example, frame 9 might be marked as the "Contact Point".
    pub fn pose_markers(&self) -> &HashMap<u16, String> {
        &self.pose_markers
    }

    /// See [`Action.method#pose_markers`]
    pub fn set_pose_markers(&mut self, pose_markers: HashMap<u16, String>) {
        self.pose_markers = pose_markers;
    }
}
