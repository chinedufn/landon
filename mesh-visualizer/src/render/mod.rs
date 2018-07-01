use assets::Assets;
use blender_mesh::BlenderMesh;
use cgmath;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;
use shader::ShaderSystem;
use shader::ShaderType;
use std::f32::consts::PI;
use std::rc::Rc;
use web_apis::WebGLProgram;
use web_apis::WebGLRenderingContext;
use State;

pub struct Renderer {
    gl: Rc<WebGLRenderingContext>,
    assets: Assets,
    shader_sys: ShaderSystem,
    state: Rc<State>,
}

trait Render {
    fn shader_type(&self) -> ShaderType;
    fn render(&self, gl: &WebGLRenderingContext, shader_program: &WebGLProgram);
}
trait BlenderMeshRender {
    fn render_non_skinned(&self, gl: &WebGLRenderingContext, shader_program: &WebGLProgram) {}
}

struct attribute<T>(T);
struct uniform<T>(T);

// TODO: These types can probably be automatically generated based on the shader
struct NonSkinnedRender {
    aVertexPos: attribute<Vec<f32>>,
    aVertexNormal: attribute<Vec<f32>>,
    uMVMatrix: uniform<Vec<f32>>,
}

impl Render for BlenderMesh {
    fn shader_type(&self) -> ShaderType {
        ShaderType::NonSkinned
    }
    fn render(&self, gl: &WebGLRenderingContext, shader_program: &WebGLProgram) {
        if let Some(_) = self.armature_name {
            // TODO: Render skinned mesh in this case
            self.render_non_skinned(&gl, &shader_program);
        } else {
            self.render_non_skinned(&gl, &shader_program);
        }
    }
}

impl BlenderMeshRender for BlenderMesh {
    fn render_non_skinned(&self, gl: &WebGLRenderingContext, shader_program: &WebGLProgram) {
        let vert_pos_attrib = gl.get_attrib_location(&shader_program, "aVertexPos");
        gl.enable_vertex_attrib_array(vert_pos_attrib);

        let vert_normal_attrib = gl.get_attrib_location(&shader_program, "aVertexNormal");
        gl.enable_vertex_attrib_array(vert_normal_attrib);

        // Temporarily using u16's until I can get GLbitfield / Glenum etc working
        let color_buffer_bit = 16384;
        let depth_buffer_bit = 256;
        // color_buffer_bit | depth_buffer_bit
        let bitfield = 16640;

        gl.clear(bitfield);

        let fovy = cgmath::Rad(PI / 3.0);
        let perspective = cgmath::perspective(fovy, 1.0, 0.1, 100.0);
        let mut p_matrix = vec_from_matrix4(&perspective);

        let model_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0));

        let mut mv_matrix = Matrix4::look_at(
            Point3::new(1.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        // TODO: Breadcrumb - add normal and point lighting to shader..

        // TODO: Multiply without new allocation
        mv_matrix = mv_matrix * model_matrix;

        let mv_matrix = vec_from_matrix4(&mv_matrix);

        let p_matrix_uni = gl.get_uniform_location(&shader_program, "uPMatrix");
        let mv_matrix_uni = gl.get_uniform_location(&shader_program, "uMVMatrix");

        gl.uniform_matrix_4fv(p_matrix_uni, false, p_matrix);
        gl.uniform_matrix_4fv(mv_matrix_uni, false, mv_matrix);

        let gl_array_buffer = 34962;
        let gl_ELEMENT_ARRAY_BUFFER = 34963;
        let gl_FLOAT = 5126;

        let static_draw = 35044;

        let vert_pos_buffer = gl.create_buffer();
        gl.bind_buffer(gl_array_buffer, &vert_pos_buffer);
        // TODO: Remove clone
        gl.buffer_f32_data(gl_array_buffer, self.vertex_positions.clone(), static_draw);
        gl.vertex_attrib_pointer(vert_pos_attrib, 3, gl_FLOAT, false, 0, 0);

        let vert_normal_buffer = gl.create_buffer();
        gl.bind_buffer(gl_array_buffer, &vert_normal_buffer);
        // TODO: Remove clone
        gl.buffer_f32_data(gl_array_buffer, self.vertex_normals.clone(), static_draw);
        gl.vertex_attrib_pointer(vert_normal_attrib, 3, gl_FLOAT, false, 0, 0);

        let index_buffer = gl.create_buffer();
        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        // TODO: Remove clone
        gl.buffer_u16_data(
            gl_ELEMENT_ARRAY_BUFFER,
            self.vertex_position_indices.clone(),
            static_draw,
        );

        let gl_TRIANGLES = 4;
        let gl_UNSIGNED_SHORT = 5123;

        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        gl.draw_elements(
            gl_TRIANGLES,
            self.vertex_position_indices.len() as u16,
            gl_UNSIGNED_SHORT,
            0,
        );
    }
}

impl Renderer {
    pub fn new(
        gl: Rc<WebGLRenderingContext>,
        assets: Assets,
        shader_sys: ShaderSystem,
        state: Rc<State>,
    ) -> Renderer {
        Renderer {
            gl,
            assets,
            shader_sys,
            state,
        }
    }

    pub fn render(&self) {
        let mesh = self.assets.meshes();
        let mesh = mesh.borrow();
        // let mesh = mesh.get(&state.current_model);
        let mesh = mesh.get("LetterF");

        if mesh.is_none() {
            return;
        }

        let mesh = mesh.unwrap();

        self.shader_sys.use_program(&ShaderType::NonSkinned);

        // TODO: Breadcrumb - armature.buffer_data() to buffer the bone quaternions into the GPU

        mesh.render(
            &self.gl,
            self.shader_sys.get_program(&ShaderType::NonSkinned),
        );
    }
}

fn vec_from_matrix4(mat4: &Matrix4<f32>) -> Vec<f32> {
    // TODO: Accept output vec instead of re-allocating
    let mut vec = vec![];

    for index in 0..16 {
        vec.push(mat4[index / 4][index % 4]);
    }

    vec
}
