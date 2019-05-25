//! The shaders that power our WebGL rendering

use crate::shader::ShaderKind::*;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::*;

pub struct ShaderSystem {
    gl: Rc<WebGlRenderingContext>,
    shaders: HashMap<ShaderKind, Shader>,
}

// TODO: Move to console_log.rs module
#[macro_export]
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[derive(Eq, PartialEq, Hash)]
pub enum ShaderKind {
    DualQuatSkin,
    NonSkinned,
    MatrixSkin,
}

pub struct Shader {
    pub program: Option<WebGlProgram>,
    // TODO: We don't need buffers for each shader.. shaders can share one set of
    // buffers. This should be a &Vec<WebGLBuffer> but need to figure out the lifetimes
    // w/ wasm-bindgen
    pub buffers: Vec<Option<WebGlBuffer>>,
}

// TODO: Breadcrumb - get our F rendering using our shader system
impl ShaderSystem {
    pub fn new(gl: Rc<WebGlRenderingContext>) -> ShaderSystem {
        let shaders = ShaderSystem::init_shaders(&gl);

        ShaderSystem { shaders, gl }
    }

    pub fn use_program(&self, shader_type: &ShaderKind) {
        self.gl
            .use_program(self.shaders.get(shader_type).unwrap().program.as_ref())
    }

    pub fn get_shader(&self, shader_type: &ShaderKind) -> Option<&Shader> {
        self.shaders.get(shader_type)
    }

    fn init_shaders(gl: &WebGlRenderingContext) -> HashMap<ShaderKind, Shader> {
        let mut shaders = HashMap::new();

        let dual_quat_vertex = include_str!("./dual-quat-vertex.glsl");
        let dual_quat_fragment = include_str!("dual-quat-fragment.glsl");
        let dual_quat_program =
            ShaderSystem::create_shader_program(&gl, dual_quat_vertex, dual_quat_fragment);

        let non_skinned_vertex = include_str!("./non-skinned-vertex.glsl");
        let non_skinned_fragment = include_str!("./non-skinned-fragment.glsl");
        let non_skinned_program =
            ShaderSystem::create_shader_program(&gl, non_skinned_vertex, non_skinned_fragment);

        let buffers = vec![
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
        ];

        shaders.insert(
            DualQuatSkin,
            Shader {
                program: Some(dual_quat_program.unwrap()),
                buffers,
            },
        );

        let buffers = vec![
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
            gl.create_buffer(),
        ];

        shaders.insert(
            NonSkinned,
            Shader {
                program: Some(non_skinned_program.unwrap()),
                buffers,
            },
        );

        shaders
    }

    // TODO: breadcrumb -> based on shader type use the right vert / frag shader
    fn create_shader_program(
        gl: &WebGlRenderingContext,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<WebGlProgram, JsValue> {
        let vert_shader = compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, vertex_shader)?;
        let frag_shader =
            compile_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, fragment_shader)?;
        let program = link_program(&gl, &vert_shader, &frag_shader)?;

        Ok(program)
    }
}

/// Create a shader program using the WebGL APIs
fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| "Could not create shader".to_string())?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".to_string()))
    }
}

/// Link a shader program using the WebGL APIs
fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| "Unable to create shader program".to_string())?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);

    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program".to_string()))
    }
}
