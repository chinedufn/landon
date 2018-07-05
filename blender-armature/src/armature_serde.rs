use serde::Serialize;
use serde::Deserialize;

use serde::ser::Serializer;
use serde::Deserializer;
use std::collections::HashMap;
use Bone;


pub fn deserialize_actions<'de, D>(deserializer: D)
                                      -> Result<HashMap<String, HashMap<String, Vec<Bone>>>, D::Error>
    where D: Deserializer<'de>
{
    Option::deserialize(deserializer).map(From::from)
}