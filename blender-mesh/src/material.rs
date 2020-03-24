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
#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct PrincipledBSDF {
    /// [r, g, b]
    base_color: MaterialInput<[f32; 3], String>,
    /// roughness
    roughness: MaterialInput<f32, (String, Channel)>,
    /// metallic
    metallic: MaterialInput<f32, (String, Channel)>,
    /// The filename for the material's normal map
    normal_map: Option<String>,
}

/// An input to a material property.
///
/// This can either be some uniform value that will get used across all vertices / fragments
/// in your shader, or a texture.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MaterialInput<U, I> {
    /// Some value that is uniform across all vertices / fragments in your mesh.
    Uniform(U),
    /// The name of the texture image (excluding the full path) from an image texture node
    /// in Blender's material editor.
    ///
    /// So a texture stored at /Users/me/hello-world.png
    /// becomes MaterialInput::Texture("hello-world.png".to_string())
    ///
    /// This means that it is important to have different texture names across all unique textures
    /// in your application.
    ///
    /// ## Note
    ///
    /// This is different from the other built in texture nodes, such as brick texture and
    /// sky texture. We do not currently support these. If these would be useful for you,
    /// open an issue!
    ///
    /// ## Examples
    ///
    /// ```
    /// // Metalness can be read from the green channel of metal.jpg
    /// use blender_mesh::{MaterialInput, Channel};
    /// let metalness: MaterialInput<f32, (String, Channel)> =
    ///     MaterialInput::ImageTexture((String::from("metal.jpg"), Channel::Green));
    /// ```
    ImageTexture(I),
}

/// An individual channel within an image.
/// Red, Green, or Blue.
#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum Channel {
    #[serde(rename = "R")]
    Red,
    #[serde(rename = "G")]
    Green,
    #[serde(rename = "B")]
    Blue,
}

impl<U, I> Default for MaterialInput<U, I>
where
    U: Default,
{
    fn default() -> Self {
        MaterialInput::Uniform(U::default())
    }
}

impl PrincipledBSDF {
    /// The base_color of the material.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.diffuse_color
    #[inline]
    pub fn base_color(&self) -> &MaterialInput<[f32; 3], String> {
        &self.base_color
    }

    /// The roughness of the material.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.roughness
    #[inline]
    pub fn roughness(&self) -> &MaterialInput<f32, (String, Channel)> {
        &self.roughness
    }

    /// How metallic the material is. Most materials should be 0.0 or 1.0.
    ///
    /// https://docs.blender.org/api/blender2.8/bpy.types.Material.html#bpy.types.Material.metallic
    #[inline]
    pub fn metallic(&self) -> &MaterialInput<f32, (String, Channel)> {
        &self.metallic
    }

    /// The normal map
    #[inline]
    pub fn normal_map(&self) -> Option<&String> {
        self.normal_map.as_ref()
    }
}
