use crate::{Action, Bone, BoneKeyframe, SortedKeyframes};
use nalgebra::DualQuaternion;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub const BONE_IDX: u8 = 123;

pub fn action_name() -> String {
    "Some Action Name".to_string()
}

pub fn action_with_keyframes(keyframes: Vec<BoneKeyframe>) -> HashMap<String, Action, RandomState> {
    let mut actions = HashMap::new();
    // let mut k = HashMap::new();
    // k.insert(BONE_IDX, keyframes);
    actions.insert(action_name(), Action::new_with_keyframes(keyframes));

    actions
}

pub fn bone_dual_quat_identity() -> Bone {
    Bone::DualQuat(DualQuaternion::identity())
}
