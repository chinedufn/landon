use crate::BlenderMesh;
use std::collections::HashMap;

/// Material data for a mesh
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub struct Material {
    /// [r, g, b]
    diffuse_color: [f32; 3],
    /// [r, g, b]
    specular_color: [f32; 3],
    /// Shininess
    specular_intensity: f32,
    /// Transparency
    alpha: f32,
}

impl Material {
    /// The diffuse_color of the material.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.diffuse_color
    #[inline]
    pub fn diffuse_color(&self) -> &[f32; 3] {
        &self.diffuse_color
    }

    /// The specular_color of the material
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.specular_color
    #[inline]
    pub fn specular_color(&self) -> &[f32; 3] {
        &self.specular_color
    }

    /// The shininess of the material, from 0 to 1
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.specular_intensity
    #[inline]
    pub fn specular_intensity(&self) -> f32 {
        self.specular_intensity
    }

    /// The transparency of the material
    #[inline]
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
}

impl BlenderMesh {
    /// Get the materials for this mesh, indexed by their name
    pub fn materials(&self) -> &HashMap<String, Material> {
        &self.materials
    }
}
