use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::SortedKeyframes;

struct SortedKeyframesVisitor;

impl<'de> Deserialize<'de> for SortedKeyframes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SortedKeyframesVisitor)
    }
}

impl<'de> Visitor<'de> for SortedKeyframesVisitor {
    type Value = SortedKeyframes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A of `BoneKeyframe`s")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut keyframes = Vec::with_capacity(1);

        while let Some(value) = seq.next_element()? {
            keyframes.push(value);
        }

        Ok(SortedKeyframes::new(keyframes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that we can deserialize keyframes
    #[test]
    fn deserialize() {
        let keyframes: SortedKeyframes = serde_yaml::from_str(
            r#"
- frame: 5
  bone: {DualQuat: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
- frame: 2
  bone: {DualQuat: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
"#,
        )
        .unwrap();

        assert_eq!(keyframes.0[0].frame(), 2);
        assert_eq!(keyframes.0[1].frame(), 5);
    }
}
