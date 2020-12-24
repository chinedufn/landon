use std::collections::HashMap;
use std::ops::Deref;

use crate::serialize_hashmap_deterministic;

pub use self::bone_keyframe::*;
pub use self::sorted_keyframes::*;

mod bone_keyframe;
mod sample;
mod sorted_keyframes;

/// The keyframes for the transformations for a bone
///
/// TODO: Custom deserialize to guarantee that the frame_range_inclusive is correct by checking
///  against all of the keyframes
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct BoneKeyframes {
    frame_range_inclusive: Option<(u16, u16)>,
    #[serde(serialize_with = "serialize_hashmap_deterministic")]
    keyframes: HashMap<u8, SortedKeyframes>,
}

impl BoneKeyframes {
    /// Create an empty set of bone keyframes.
    pub fn new() -> BoneKeyframes {
        Self::default()
    }

    /// Create a new set of bone keyframes
    #[cfg(test)]
    pub fn new_with_keyframes(mut keyframes: Vec<BoneKeyframe>) -> Self {
        use crate::test_util::BONE_IDX;
        let mut map = HashMap::new();
        map.insert(BONE_IDX, SortedKeyframes::new(keyframes));

        let mut keyframes = BoneKeyframes {
            frame_range_inclusive: None,
            keyframes: map,
        };

        keyframes.update_frame_range_inclusive();

        keyframes
    }

    pub fn smallest_frame(&self) -> Option<u16> {
        Some(self.frame_range_inclusive?.0)
    }

    pub fn largest_frame(&self) -> Option<u16> {
        Some(self.frame_range_inclusive?.1)
    }

    pub fn frame_duration(&self) -> Option<u16> {
        Some(self.largest_frame()? - self.smallest_frame()?)
    }

    pub fn frame_range_inclusive(&self) -> Option<(u16, u16)> {
        self.frame_range_inclusive
    }

    /// Add a trnasformation keyframe for a bone.
    pub fn insert_bone_keyframe(&mut self, bone_idx: u8, keyframe: BoneKeyframe) {
        let keyframes = self.keyframes.entry(bone_idx).or_default();

        keyframes.push(keyframe);

        keyframes.sort_by(|a, b| a.frame().cmp(&b.frame()));

        self.update_frame_range_inclusive();
    }

    fn update_frame_range_inclusive(&mut self) {
        let mut frame_range_inclusive = None;

        let mut frames_found = false;

        let mut smallest_frame = u16::max_value();
        let mut largest_frame = 0;

        for (_, keyframes) in self.keyframes.iter_mut() {
            for keyframe in keyframes.iter() {
                frames_found = true;

                smallest_frame = smallest_frame.min(keyframe.frame());
                largest_frame = largest_frame.max(keyframe.frame());
            }

            keyframes.sort_by(|a, b| a.frame().cmp(&b.frame()));
        }

        if frames_found {
            frame_range_inclusive = Some((smallest_frame, largest_frame));
        }

        self.frame_range_inclusive = frame_range_inclusive;
    }
}

impl Deref for BoneKeyframes {
    type Target = HashMap<u8, SortedKeyframes>;

    fn deref(&self) -> &Self::Target {
        &self.keyframes
    }
}

impl BoneKeyframes {
    pub(crate) fn keyframes_mut(&mut self) -> &mut HashMap<u8, SortedKeyframes> {
        &mut self.keyframes
    }
}
