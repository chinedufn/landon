use std::collections::BTreeMap;

use crate::{BlenderArmature, Bone, JointIndicesRef, SampleDesc};

impl BlenderArmature {
    pub(super) fn sample_action(
        &self,
        action_name: &str,
        joint_indices: JointIndicesRef,
        sample_desc: SampleDesc,
    ) -> BTreeMap<u8, Bone> {
        let joint_indices = match joint_indices {
            JointIndicesRef::All => unimplemented!("TODO"),
            JointIndicesRef::Some(joint_indices) => joint_indices,
        };

        let mut bones = BTreeMap::new();

        for joint_idx in joint_indices {
            let bone_keyframes = self
                .bone_space_actions
                .get(action_name)
                .unwrap()
                .bone_keyframes();

            let bone = bone_keyframes.sample(*joint_idx, sample_desc);

            bones.insert(*joint_idx, bone);
        }

        bones
    }
}
