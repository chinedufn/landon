use crate::action::action_keyframes::ActionKeyframes;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

impl Serialize for ActionKeyframes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.keyframes.len()))?;

        for k in self.keyframes.iter() {
            seq.serialize_element(k)?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keyframe;

    /// Verify that we properly serialize action keyframes
    #[test]
    fn serialize() {
        let action_keyframes = ActionKeyframes::new(vec![Keyframe::new(5, vec![])]);
        let serialized = serde_yaml::to_string(&action_keyframes).unwrap();

        assert_eq!(
            serialized,
            r#"---
- frame: 5
  bones: []"#
        )
    }
}
