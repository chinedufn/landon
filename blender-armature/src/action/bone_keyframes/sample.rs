use crate::action::get_surrounding_keyframes;
use crate::{interpolate_bone, Bone, BoneKeyframes, SampleDesc};

impl BoneKeyframes {
    /// Sample the bone transforms
    pub fn sample(&self, joint_idx: u8, sample_desc: SampleDesc) -> Bone {
        let keyframes = self.keyframes.get(&joint_idx).unwrap();

        let (lowest_keyframe, highest_keyframe) = self.frame_range_inclusive().unwrap();

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
            (key_time_to_sample - action_lower_keyframe.frame() as f32)
                / (action_upper_keyframe.frame() - action_lower_keyframe.frame()) as f32
        };

        let lower_bone = action_lower_keyframe.bone();
        let upper_bone = action_upper_keyframe.bone();

        interpolate_bone(lower_bone, upper_bone, percent_elapsed_into_keyframe)
    }
}
