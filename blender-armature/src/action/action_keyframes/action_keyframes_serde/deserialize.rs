use std::fmt;

use serde::de::{self, SeqAccess, Visitor};

use crate::action::action_keyframes::ActionKeyframes;
use serde::{Deserialize, Deserializer};
struct ActionKeyframesVisitor;

impl<'de> Deserialize<'de> for ActionKeyframes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ActionKeyframesVisitor)
    }
}

impl<'de> Visitor<'de> for ActionKeyframesVisitor {
    type Value = ActionKeyframes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A non-empty sequence of `Keyframe`s")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut keyframes = Vec::with_capacity(1);

        while let Some(value) = seq.next_element()? {
            keyframes.push(value);
        }

        // This allows lowest/smallest frame caches to not have to be Option<u16>
        // If in the future we need to support empty keyframe lists we can remove this requirement
        // and just make our largest/smallest keyframe cache and Option<(u16, u16)>
        if keyframes.len() == 0 {
            return Err(de::Error::custom(
                "sequence must contain at least one keyframe",
            ));
        }

        Ok(ActionKeyframes::new(keyframes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Verify that there is an error if there are no keyframes.
    #[test]
    fn empty_list() {
        assert!(serde_yaml::from_str::<ActionKeyframes>(r#"[]"#).is_err());
    }

    /// Verify that we can deserialize keyframes
    #[test]
    fn deserialize() {
        let action_keyframes: ActionKeyframes = serde_yaml::from_str(
            r#"
- frame: 5
  bones: []
- frame: 2
  bones: []
"#,
        )
        .unwrap();

        assert_eq!(action_keyframes.smallest_frame, 2);
        assert_eq!(action_keyframes.largest_frame, 5);
    }
}
