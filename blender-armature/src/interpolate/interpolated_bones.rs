use crate::Bone;
use std::collections::BTreeMap;

/// Blend from the start bones towards the ending bones.
///
/// When the interpolation parameter is 0.0 the start bone is used.
/// At 1.0 the end bone is used.
pub fn blend_towards_bones(
    start: &BTreeMap<u8, Bone>,
    end: &BTreeMap<u8, Bone>,
    interp_param: f32,
) -> BTreeMap<u8, Bone> {
    start
        .iter()
        .zip(end.iter())
        .map(
            |((prev_joint_idx, prev_action_bone), (cur_joint_idx, cur_action_bone))| {
                // TODO: We were using a hashmap where the iteration order isn't guaranteed and hence we would hit this condition.
                //  Really just need to refactor all of landon now that we're much more experienced with Rust.
                if prev_joint_idx != cur_joint_idx {
                    panic!("We do not currently support the current action having different joints than the previous action");
                }

                let prev = prev_action_bone.as_slice();
                let mut prev_action_bone: [f32; 8] = [0.0; 8];
                prev_action_bone.copy_from_slice(prev);

                // Get the dot product of the start and end rotation quaternions. If the
                // dot product is negative we negate the first dual quaternion in order to
                // ensure the shortest path rotation.
                //
                // http://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
                // https://github.com/chinedufn/skeletal-animation-system/blob/9ae17c5b23759f7147bf7c464564e32a09e619ef/src/blend-dual-quaternions.js#L59
                if dot_product(&prev_action_bone, cur_action_bone.as_slice()) < 0.0 {
                    prev_action_bone[0] = -prev_action_bone[0];
                    prev_action_bone[1] = -prev_action_bone[1];
                    prev_action_bone[2] = -prev_action_bone[2];
                    prev_action_bone[3] = -prev_action_bone[3];
                    prev_action_bone[4] = -prev_action_bone[4];
                    prev_action_bone[5] = -prev_action_bone[5];
                    prev_action_bone[6] = -prev_action_bone[6];
                    prev_action_bone[7] = -prev_action_bone[7];
                }

                let _new_bone = [0.0; 8];

                let new_bone = interpolate_bone(&Bone::DualQuat(prev_action_bone), &cur_action_bone, interp_param);

                (*cur_joint_idx, new_bone)
            },
        )
        .collect()
}

pub(crate) fn interpolate_bone(start_bone: &Bone, end_bone: &Bone, amount: f32) -> Bone {
    match start_bone {
        &Bone::DualQuat(ref start_dual_quat) => match end_bone {
            &Bone::DualQuat(ref end_dual_quat) => {
                let mut interpolated_dual_quat: [f32; 8] = [0.0; 8];

                for index in 0..8 {
                    let start = start_dual_quat[index];
                    let end = end_dual_quat[index];
                    interpolated_dual_quat[index] = (end - start) * amount + start;
                }

                Bone::DualQuat(interpolated_dual_quat)
            }
            _ => panic!(
                "You may only interpolate bones of the same type. Please convert\
                 your end bone into a dual quaternion before interpolating"
            ),
        },
        &Bone::Matrix(ref _matrix) => unimplemented!(),
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}
