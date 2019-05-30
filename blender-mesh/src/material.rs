use crate::BlenderMesh;
use std::collections::HashMap;

/// Material data for a mesh
///
/// # Blender
///
/// When exporting from Blender we read this data from the first Principled BSDF node in the
/// node editor for the material
///
/// https://docs.blender.org/manual/en/latest/render/cycles/nodes/types/shaders/principled.html
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub struct PrincipledBSDF {
    /// [r, g, b]
    base_color: [f32; 3],
    /// roughness
    roughness: f32,
    /// metallic
    metallic: f32,
}

impl PrincipledBSDF {
    /// The base_color of the material.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.diffuse_color
    #[inline]
    pub fn base_color(&self) -> &[f32; 3] {
        &self.base_color
    }

    /// The roughness of the material.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.roughness
    #[inline]
    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    /// How metallic the material is. Most materials should be 0.0 or 1.0.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.metallic
    #[inline]
    pub fn metallic(&self) -> f32 {
        self.metallic
    }
}

impl BlenderMesh {
    /// Get the materials for this mesh, indexed by their name
    pub fn materials(&self) -> &HashMap<String, PrincipledBSDF> {
        &self.materials
    }
}
