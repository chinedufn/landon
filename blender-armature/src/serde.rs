use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Used to serialize the same HashMap consistently every time.
///
/// Useful for generating reproducible builds when serializing assets in an asset pipeline.
pub fn serialize_hashmap_deterministic<S, K, V>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize + Hash + Eq + Ord,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
