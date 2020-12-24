use crate::BoneKeyframe;
use std::cmp::Ordering;
use std::ops::Deref;
use std::slice::IterMut;

mod deserialize;

/// Keyframes sorted in ascending frame order
#[derive(Debug, PartialEq, Serialize, Default, Clone)]
pub struct SortedKeyframes(Vec<BoneKeyframe>);

impl SortedKeyframes {
    /// Create a new SortedKeyframes.
    ///
    /// The passed in keyframes will get be sorted.
    pub fn new(keyframes: Vec<BoneKeyframe>) -> Self {
        let mut keys = SortedKeyframes(keyframes);

        keys.sort_ascending();

        keys
    }

    fn sort_ascending(&mut self) {
        self.0.sort_by(|a, b| a.frame().cmp(&b.frame()));
    }
}

impl SortedKeyframes {
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, BoneKeyframe> {
        self.0.iter_mut()
    }

    pub(crate) fn push(&mut self, bone_keyframe: BoneKeyframe) {
        self.0.push(bone_keyframe)
    }

    pub(crate) fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&BoneKeyframe, &BoneKeyframe) -> Ordering,
    {
        self.0.sort_by(compare);
    }
}

impl Deref for SortedKeyframes {
    type Target = Vec<BoneKeyframe>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
