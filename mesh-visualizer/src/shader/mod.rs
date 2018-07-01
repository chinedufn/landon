//! The shaders that power our WebGL rendering

use shader::ShaderType::*;
use std::collections::HashMap;
use std::rc::Rc;
use web_apis::log;
use web_apis::WebGLProgram;
use web_apis::WebGLRenderingContext;

pub struct ShaderSystem {
    gl: Rc<WebGLRenderingContext>,
    shaders: HashMap<ShaderType, Shader>,
}

// TODO: Move to console_log.rs module
#[macro_export]
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[derive(Eq, PartialEq, Hash)]
pub enum ShaderType {
    DualQuatSkin,
    NonSkinned,
    MatrixSkin,
}

pub struct Shader {
    program: WebGLProgram,
}

// TODO: Breadcrumb - get our F rendering using our shader system
impl ShaderSystem {
    pub fn new(gl: Rc<WebGLRenderingContext>) -> ShaderSystem {
        let mut shaders = ShaderSystem::init_shaders(&gl);

        ShaderSystem { shaders, gl }
    }

    pub fn use_program(&self, shader_type: &ShaderType) {
        self.gl
            .use_program(&self.shaders.get(shader_type).unwrap().program)
    }

    pub fn get_program<'a>(&'a self, shader_type: &ShaderType) -> &'a WebGLProgram {
        &self.shaders.get(shader_type).unwrap().program
    }

    fn init_shaders(gl: &WebGLRenderingContext) -> HashMap<ShaderType, Shader> {
        let mut shaders = HashMap::new();

        let dual_quat_vertex = include_str!("./dual-quat-vertex.glsl");
        let dual_quat_fragment = include_str!("dual-quat-fragment.glsl");
        let dual_quat_program =
            ShaderSystem::create_shader_program(&gl, dual_quat_vertex, dual_quat_fragment);

        let non_skinned_vertex = include_str!("./non-skinned-vertex.glsl");
        let non_skinned_fragment = include_str!("./non-skinned-fragment.glsl");
        let non_skinned_program =
            ShaderSystem::create_shader_program(&gl, non_skinned_vertex, non_skinned_fragment);

        shaders.insert(
            DualQuatSkin,
            Shader {
                program: dual_quat_program,
            },
        );
        shaders.insert(
            NonSkinned,
            Shader {
                program: non_skinned_program,
            },
        );

        shaders
    }

    // TODO: breadcrumb -> based on shader type use the right vert / frag shader
    fn create_shader_program(
        gl: &WebGLRenderingContext,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> WebGLProgram {
        // gl.FRAGMENT_SHADER
        let gl_fragment_shader = 35632;
        // gl.VERTEX_SHADER
        let gl_vertex_shader = 35633;

        let frag_shader = gl.create_shader(gl_fragment_shader);
        let vert_shader = gl.create_shader(gl_vertex_shader);

        gl.shader_source(&vert_shader, vertex_shader);
        gl.shader_source(&frag_shader, fragment_shader);

        gl.compile_shader(&vert_shader);
        gl.compile_shader(&frag_shader);

        let vert_log = gl.get_shader_info_log(&vert_shader);
        if vert_log.len() > 0 {
            clog!("Vertex shader compilation errors: {}", vert_log);
        }

        let frag_log = gl.get_shader_info_log(&frag_shader);
        if frag_log.len() > 0 {
            clog!("Fragment shader compilation errors: {}", frag_log);
        }

        let shader_program = gl.create_program();
        gl.attach_shader(&shader_program, &frag_shader);
        gl.attach_shader(&shader_program, &vert_shader);
        gl.link_program(&shader_program);

        shader_program
    }
}
