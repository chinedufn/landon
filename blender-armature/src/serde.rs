use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

/// Used to serialize the same HashMap consistently every time.
///
/// Useful for generating reproducible builds when serializing assets in an asset pipeline.
pub fn serialize_hashmap_deterministic<S, T>(
    value: &HashMap<String, T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
