use std::collections::BTreeMap;

use crate::{BlenderArmature, Bone, SampleDesc};

impl BlenderArmature {
    /// FIXME: Refactor
    pub(super) fn interpolate_action(
        &self,
        action_name: &str,
        sample_desc: SampleDesc,
    ) -> BTreeMap<u8, Bone> {
        self.actions
            .get(action_name)
            .unwrap()
            .keyframes()
            .sample(sample_desc)
    }
}
