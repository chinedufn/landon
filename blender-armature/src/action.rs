use std::collections::HashMap;

pub use self::action_keyframes::*;
pub use self::bone_keyframes::*;
use crate::Keyframe;

type Frame = u16;

mod action_keyframes;
mod bone_keyframes;

/// A set of keyframes along with metadata such as pose markers.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct Action {
    // TODO: Remove `keyframes` and replace them with keyframes.
    pub(super) bone_keyframes: BoneKeyframes,
    #[serde(default)]
    pose_markers: HashMap<Frame, String>,
    pub(super) keyframes: Vec<Keyframe>,
}

impl Action {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Action {
            bone_keyframes: BoneKeyframes::default(),
            pose_markers: HashMap::new(),
            keyframes: vec![],
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_keyframes(keyframes: HashMap<u8, Vec<SortedKeyframes>>) -> Self {
        Action {
            bone_keyframes: BoneKeyframes::new_with_keyframes(keyframes),
            pose_markers: HashMap::new(),
            keyframes: vec![],
        }
    }

    /// The world space transform keyframes for each bone
    pub fn bone_keyframes(&self) -> &BoneKeyframes {
        &self.bone_keyframes
    }

    /// Add a trnasformation keyframe for a bone.
    pub fn insert_bone_keyframe(&mut self, bone_idx: u8, keyframe: BoneKeyframe) {
        self.bone_keyframes.insert_bone_keyframe(bone_idx, keyframe);
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

    /// The smallest frame
    pub fn smallest_frame(&self) -> u16 {
        self.bone_keyframes.frame_range_inclusive().unwrap().0
    }

    /// The largest frame
    pub fn largest_frame(&self) -> u16 {
        self.bone_keyframes.frame_range_inclusive().unwrap().1
    }

    /// Last frame - first frame
    pub fn frame_duration(&self) -> u16 {
        self.bone_keyframes.frame_duration().unwrap()
    }
}

// pub(crate)
impl Action {
    /// We use crate visibility to prevent users from being able to modify keyframes without
    /// updating the cached smallest/largest frame number.
    ///
    /// See [`Action.method#keyframes`]
    pub(crate) fn keyframes_mut(&mut self) -> &mut HashMap<u8, SortedKeyframes> {
        self.bone_keyframes.keyframes_mut()
    }
}
