use crate::Bone;
use nalgebra::DualQuaternion;
use std::collections::BTreeMap;

/// Blend from the start bones towards the ending bones.
///
/// TODO: Delete. We now favor blending once at a time since this makes for a simpler API with
///  fewer allocations
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


                let new_bone = interpolate_bone(*prev_action_bone, *cur_action_bone, interp_param);

                (*cur_joint_idx, new_bone)
            },
        )
        .collect()
}

/// Interpolate from the start to the end bone using the given amount between [0.0, 1.0] inclusive.
///
/// When the interpolation parameter is 0.0 the start bone is used.
/// At 1.0 the end bone is used.
///
/// # Panics
///
/// Panics of the `amount < 0.0 || amount > 1.0`
pub fn interpolate_bone(start_bone: Bone, end_bone: Bone, amount: f32) -> Bone {
    match start_bone {
        Bone::DualQuat(start) => match end_bone {
            Bone::DualQuat(mut end) => Bone::DualQuat(interpolate_dual_quats(start, end, amount)),
            _ => panic!(
                r#"You may only interpolate bones of the same type. Please convert
your end bone into a dual quaternion before interpolating"#
            ),
        },
        Bone::Matrix(_matrix) => unimplemented!(),
    }
}

/// Interpolate from the start to the end bone using the given amount between [0.0, 1.0] inclusive.
///
/// When the interpolation parameter is 0.0 the start bone is used.
/// At 1.0 the end bone is used.
///
/// # Panics
///
/// Panics of the `amount < 0.0 || amount > 1.0`
///
/// TODO: Move this into nalgebra
pub fn interpolate_dual_quats(
    start: DualQuaternion<f32>,
    mut end: DualQuaternion<f32>,
    amount: f32,
) -> DualQuaternion<f32> {
    // Get the dot product of the start and end rotation quaternions. If the
    // dot product is negative we negate one of the dual quaternions in order to
    // ensure the shortest path rotation.
    //
    // http://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
    if start.real.dot(&end.real) < 0.0 {
        end = end * -1.;
    }

    start + ((end - start) * amount)
}
