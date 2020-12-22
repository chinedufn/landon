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

        self.actions
            .get(action_name)
            .unwrap()
            .keyframes()
            .sample(joint_indices, sample_desc)
    }
}
