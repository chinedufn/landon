use std::collections::BTreeMap;

use crate::{interpolate_bone, ActionKeyframes, Bone};

pub use self::joint_indices::*;
pub use self::sample_desc::*;
use self::surrounding_keyframes::get_surrounding_keyframes;

mod joint_indices;
mod sample_desc;
mod surrounding_keyframes;

impl ActionKeyframes {
    /// Sample the bones based on the amount of elapsed time at the given framerate.
    ///
    /// See [`ActionSettings.elapsed_time`]
    pub fn sample(&self, joint_indices: &[u8], sample_desc: SampleDesc) -> BTreeMap<u8, Bone> {
        let mut interpolated_bones = BTreeMap::new();

        if joint_indices.len() == 0 {
            return interpolated_bones;
        }

        let keyframes = self.keyframes();

        let (lowest_keyframe, highest_keyframe) =
            (self.smallest_frame as f32, self.largest_frame as f32);

        let mut frames_elapsed = sample_desc.frame_offset.get();

        let mut key_time_to_sample = lowest_keyframe as f32 + frames_elapsed;

        let action_duration = (highest_keyframe - lowest_keyframe) as f32;

        if frames_elapsed > action_duration {
            if sample_desc.should_loop {
                frames_elapsed = frames_elapsed % action_duration;
            } else {
                frames_elapsed = action_duration;
            }

            key_time_to_sample = lowest_keyframe as f32 + frames_elapsed;
        }

        let (action_lower_keyframe, action_upper_keyframe) =
            get_surrounding_keyframes(keyframes, key_time_to_sample);

        let percent_elapsed_into_keyframe = if action_lower_keyframe == action_upper_keyframe {
            0.0
        } else {
            (key_time_to_sample - action_lower_keyframe.frame as f32)
                / (action_upper_keyframe.frame - action_lower_keyframe.frame) as f32
        };

        for joint_index in joint_indices.iter() {
            let joint_index = *joint_index;

            let lower_bone = &action_lower_keyframe.bones[joint_index as usize];
            let upper_bone = &action_upper_keyframe.bones[joint_index as usize];

            let interpolated_bone =
                interpolate_bone(&lower_bone, &upper_bone, percent_elapsed_into_keyframe);
            interpolated_bones.insert(joint_index, interpolated_bone);
        }

        interpolated_bones
    }
}
