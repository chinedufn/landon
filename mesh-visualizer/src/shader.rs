//! The shaders that power our WebGL rendering

use std::collections::HashMap;
use web_apis::WebGLProgram;

pub struct ShaderSystem {
    shaders: HashMap<ShaderType, Shader>,
}

pub enum ShaderType {
    DualQuatSkin,
    NonSkinned,
    MatrixSkin,
}

pub struct Shader {
    program: WebGLProgram,
}
