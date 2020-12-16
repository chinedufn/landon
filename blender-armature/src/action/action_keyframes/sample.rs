use crate::{interpolate_bone, ActionKeyframes, Bone, Keyframe};
use std::collections::BTreeMap;

mod sample_desc;
mod surrounding_keyframes;
pub use self::sample_desc::*;
use self::surrounding_keyframes::get_surrounding_keyframes;

impl ActionKeyframes {
    /// Sample the bones based on the amount of elapsed time at the given framerate.
    ///
    /// See [`ActionSettings.elapsed_time`]
    pub fn sample(&self, sample_desc: SampleDesc) -> BTreeMap<u8, Bone> {
        let mut interpolated_bones = BTreeMap::new();

        if sample_desc.joint_indices.len() == 0 {
            return interpolated_bones;
        }

        let keyframes = self.keyframes();

        let (lowest_keyframe, highest_keyframe) = self.find_lowest_and_highest_keyframe();

        let mut frames_elapsed = sample_desc.frame_offset.get();

        let mut key_time_to_sample = lowest_keyframe.frame as f32 + frames_elapsed;

        let action_duration = (highest_keyframe.frame - lowest_keyframe.frame) as f32;

        if frames_elapsed > action_duration {
            if sample_desc.should_loop {
                frames_elapsed = frames_elapsed % action_duration;
            } else {
                frames_elapsed = action_duration;
            }

            key_time_to_sample = lowest_keyframe.frame as f32 + frames_elapsed;
        }

        let (action_lower_keyframe, action_upper_keyframe) =
            get_surrounding_keyframes(keyframes, key_time_to_sample);

        let percent_elapsed_into_keyframe = if action_lower_keyframe == action_upper_keyframe {
            0.0
        } else {
            (key_time_to_sample - action_lower_keyframe.frame as f32)
                / (action_upper_keyframe.frame - action_lower_keyframe.frame) as f32
        };

        for joint_index in sample_desc.joint_indices.iter() {
            let joint_index = *joint_index;

            let lower_bone = &action_lower_keyframe.bones[joint_index as usize];
            let upper_bone = &action_upper_keyframe.bones[joint_index as usize];

            let interpolated_bone =
                interpolate_bone(&lower_bone, &upper_bone, percent_elapsed_into_keyframe);
            interpolated_bones.insert(joint_index, interpolated_bone);
        }

        interpolated_bones
    }

    fn find_lowest_and_highest_keyframe<'a>(&'a self) -> (&'a Keyframe, &'a Keyframe) {
        let keyframes = self.keyframes();

        let mut lowest_keyframe = u16::max_value();
        let mut lowest_keyframe_idx = 0;

        let mut highest_keyframe = 0;
        let mut highest_keyframe_idx = 0;

        for (index, keyframe) in keyframes.iter().enumerate() {
            if keyframe.frame < lowest_keyframe {
                lowest_keyframe = keyframe.frame;
                lowest_keyframe_idx = index;
            } else if keyframe.frame > highest_keyframe {
                highest_keyframe = keyframe.frame;
                highest_keyframe_idx = index;
            }
        }

        (
            &keyframes[lowest_keyframe_idx],
            &keyframes[highest_keyframe_idx],
        )
    }
}
