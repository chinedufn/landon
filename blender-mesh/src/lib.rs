//! Blender files can have meshes such as circles, cubes, cylinders, a dragon or any other
//! 3D shape.
//!
//! A mesh can be represented as a group of vertices and data about those vertices, such as their
//! normals or UV coordinates.
//!
//! Meshes can also have metadata, such as the name of it's parent armature (useful for vertex
//! skinning).
//!
//! blender-mesh-to-json seeks to be a well tested, well documented exporter for blender mesh
//! metadata.
//!
//! You can write data to stdout or to a file. At the onset it will be geared towards @chinedufn's
//! needs - but if you have needs that aren't met feel very free to open an issue.
//!
//! @see https://docs.blender.org/manual/en/dev/modeling/meshes/introduction.html - Mesh Introduction
//! @see https://github.com/chinedufn/blender-actions-to-json - Exporting blender armatures / actions

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

pub use self::combine_indices::CreateSingleIndexConfig;
pub use self::export::*;
pub use crate::bounding_box::BoundingBox;
use crate::custom_property::CustomProperty;
pub use crate::material::PrincipledBSDF;
use crate::serde::serialize_hashmap_deterministic;
pub use crate::vertex_attributes::{
    BoneInfluence, MultiIndexedVertexAttributes, SingleIndexedVertexAttributes, Vertex,
    VertexAttribute,
};
pub use material::{Channel, MaterialInput};
use std::collections::HashMap;

mod bone;
mod bounding_box;
mod combine_indices;
mod custom_property;
mod export;
mod face_tangents;
mod interleave;
mod material;
mod serde;
mod triangulate;
mod vertex_attributes;
mod y_up;

mod create_mesh;

#[cfg(test)]
mod test_utils;

/// Something went wrong in the Blender child process that was trying to parse your mesh data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export meshes from a `.blend` file.
    #[fail(
        display = "There was an issue while exporting meshes: Blender stderr output: {}",
        _0
    )]
    Stderr(String),
}

/// All of the data about a mesh
///
/// TODO: Rename crate to `MeshIr`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct BlenderMesh {
    name: String,
    armature_name: Option<String>,
    bounding_box: BoundingBox,
    #[serde(alias = "attribs")]
    multi_indexed_vertex_attributes: MultiIndexedVertexAttributes,
    #[serde(default, serialize_with = "serialize_hashmap_deterministic")]
    materials: HashMap<String, PrincipledBSDF>,
    #[serde(default, serialize_with = "serialize_hashmap_deterministic")]
    custom_properties: HashMap<String, CustomProperty>,
}

impl BlenderMesh {
    /// The name of this mesh's parent armature
    pub fn armature_name(&self) -> Option<&String> {
        self.armature_name.as_ref()
    }

    /// Set the name of this mesh's parent armature
    pub fn set_armature_name(&mut self, armature_name: Option<String>) {
        self.armature_name = armature_name;
    }

    /// A map of material name to the material's data
    pub fn materials(&self) -> &HashMap<String, PrincipledBSDF> {
        &self.materials
    }

    /// A mutable map of material name to the material's data
    pub fn materials_mut(&mut self) -> &mut HashMap<String, PrincipledBSDF> {
        &mut self.materials
    }

    /// Custom properties for this mesh
    ///
    /// i.e. in Blender this might be found with `bpy.context.view_layer.objects.active.keys()`
    pub fn custom_properties(&self) -> &HashMap<String, CustomProperty> {
        &self.custom_properties
    }

    /// The smallest box that contains the entire mesh
    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    /// Set the mesh's bounding box.
    pub fn set_bounding_box(&mut self, bounding_box: BoundingBox) {
        self.bounding_box = bounding_box;
    }

    /// The name of the mesh
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Set the name of the mesh
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// Concatenate a series of vectors into one vector.
///
/// Useful for generating fake vertex data for unit tests.
///
/// ```ignore
/// assert_eq!(
///     concat_vecs!(vec![1, 2, 3], vec![4,5]),
///     vec![1, 2, 3, 4, 5]
/// );
/// ```
#[cfg(test)]
#[macro_export]
#[cfg(test)]
macro_rules! concat_vecs {
    ( $( $vec:expr),* ) => {
        {
            let mut concatenated_vec = Vec::new();
            $(
                concatenated_vec.append(&mut $vec.clone());
            )*
            concatenated_vec
        }
    }
}

#[cfg(test)]
fn indexed(
    attribute: crate::vertex_attributes::VertexAttribute<f32>,
) -> crate::vertex_attributes::IndexedAttribute {
    crate::vertex_attributes::IndexedAttribute {
        indices: vec![],
        attribute,
    }
}
