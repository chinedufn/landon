use crate::BoneKeyframe;

/// If you're sampling frame 1.5 and there are three keyframes - 0, 2, 3 the
/// surrounding keyframes are 0 and 2.
///
/// If you're sampling frame 1.5 and there are three keyframes - 10, 15, 31 the
/// surrounding keyframes are 0 and 0.
///
/// If you're sampling frame 1.5 and there are three keyframes - 0, and 1
/// then the surrounding keyframes are 1 and 1.
///
/// We assume that the keyframes are stored in ascending order.
///
/// TODO: Binary search instead of linear
pub fn get_surrounding_keyframes(
    keyframes: &Vec<BoneKeyframe>,
    current_frame: f32,
) -> (BoneKeyframe, BoneKeyframe) {
    let mut closest_lower = None;
    let mut closest_upper = None;

    for (idx, frame) in keyframes.iter().enumerate() {
        if (frame.frame() as f32) <= current_frame {
            closest_lower = Some(idx)
        }

        if (frame.frame() as f32) >= current_frame {
            closest_upper = Some(idx);
            break;
        }
    }

    if closest_upper.is_none() {
        closest_upper = closest_lower;
    } else if closest_lower.is_none() {
        closest_lower = closest_upper;
    }

    (
        keyframes[closest_lower.unwrap()],
        keyframes[closest_upper.unwrap()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::bone_dual_quat_identity;

    /// Verify that we properly determine the lower and upper keyframe to sample
    #[test]
    fn surrounding_keyframes() {
        let keyframes = vec![
            BoneKeyframe::new(2, bone_dual_quat_identity()),
            BoneKeyframe::new(5, bone_dual_quat_identity()),
            BoneKeyframe::new(8, bone_dual_quat_identity()),
        ];

        let tests = vec![
            //
            (0.0, [0, 0]),
            (4.0, [0, 1]),
            (5.0, [1, 1]),
            (7.0, [1, 2]),
            (8.0, [2, 2]),
            (9.0, [2, 2]),
        ];

        for (elapsed_frames, [expected_lower, expected_upper]) in tests {
            let (lower, upper) = get_surrounding_keyframes(&keyframes, elapsed_frames);

            assert_eq!(lower, keyframes[expected_lower]);
            assert_eq!(upper, keyframes[expected_upper]);
        }
    }
}
