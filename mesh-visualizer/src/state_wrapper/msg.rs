/// Messages to send to State that it will use to update itself.
pub enum Msg {
    /// Zoom in/out depending on if a positive or negative float is passed
    Zoom(f32),
    /// Set the current mesh to view in the model viewer
    SetCurrentMesh(String),
}
