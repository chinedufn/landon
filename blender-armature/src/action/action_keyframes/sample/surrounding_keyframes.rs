use crate::Keyframe;

/// If you're sampling time 1.5seconds and there are three keyframes, 0.0s, 1.8s, 2.2s the
/// surrounding keyframes are 0.0s and 1.8s
pub(super) fn get_surrounding_keyframes(
    keyframes: &Vec<Keyframe>,
    elapsed_frames: f32,
) -> (&Keyframe, &Keyframe) {
    let mut action_lower_keyframe = 0;
    let mut action_upper_keyframe = 0;

    let mut lowest_time_seen = -std::f32::INFINITY;
    let mut highest_time_seen = std::f32::INFINITY;

    for (index, keyframe) in keyframes.iter().enumerate() {
        let frame = keyframe.frame as f32;

        if frame <= elapsed_frames && frame >= lowest_time_seen {
            action_lower_keyframe = index;
            lowest_time_seen = frame;
        }

        if frame >= elapsed_frames && frame <= highest_time_seen {
            action_upper_keyframe = index;
            highest_time_seen = frame;
        }
    }

    (
        &keyframes[action_lower_keyframe],
        &keyframes[action_upper_keyframe],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keyframe;

    #[test]
    fn surrounding_keyframes() {
        let keyframes = vec![
            Keyframe {
                frame: 0,
                bones: vec![],
            },
            Keyframe {
                frame: 8,
                bones: vec![],
            },
            Keyframe {
                frame: 3,
                bones: vec![],
            },
        ];

        let (lower, upper) = get_surrounding_keyframes(&keyframes, 0.3);
        assert_eq!(lower, &keyframes[0]);
        assert_eq!(upper, &keyframes[2]);

        let (lower, upper) = get_surrounding_keyframes(&keyframes, 4.0);
        assert_eq!(lower, &keyframes[2]);
        assert_eq!(upper, &keyframes[1]);
    }
}
