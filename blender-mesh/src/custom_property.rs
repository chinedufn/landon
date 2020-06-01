/// A [Blender custom property][custom-property]:
/// [custom-property]: https://docs.blender.org/manual/en/latest//files/data_blocks.html#custom-properties
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub enum CustomProperty {
    Float(f32),
    Int(i32),
    String(String),
    Vec(Vec<CustomPropertyVecItem>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub enum CustomPropertyVecItem {
    Float(f32),
    Int(i32),
    String(String),
}
