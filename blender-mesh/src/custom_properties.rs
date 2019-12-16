use crate::BlenderMesh;
use std::collections::HashMap;

impl BlenderMesh {
    /// Get the custom properties for this mesh
    pub fn custom_properties(&self) -> Option<&HashMap<String, f32>> {
        self.custom_properties.as_ref()
    }

    /// Get the custom properties for this mesh mutably
    pub fn custom_properties_mut(&mut self) -> Option<&mut HashMap<String, f32>> {
        self.custom_properties.as_mut()
    }
}
