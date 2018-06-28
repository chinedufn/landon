#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate blender_mesh;
extern crate cgmath;
extern crate wasm_bindgen;

use blender_mesh::BlenderMesh;

use wasm_bindgen::prelude::*;

pub mod web_apis;

use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;
use web_apis::*;

use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;

mod assets;
use assets::Assets;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn download_mesh(mesh_name: &str, mesh_path: &str, cb: &Closure<FnMut(String, String)>);
}

#[wasm_bindgen]
pub struct App {
    /// The model that the user is currently viewing in their browser
    current_model: String,
    /// All of the models that we have downloaded and can render
    meshes: Rc<RefCell<HashMap<String, BlenderMesh>>>,
    /// A handle into the WebGL context for our canvas
    gl: Option<WebGLRenderingContext>,
    non_skinned_shader_program: Option<WebGLProgram>,
    assets: Assets
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        App {
            meshes: Rc::new(RefCell::new(HashMap::new())),
            current_model: "LetterF".to_string(),
            gl: None,
            non_skinned_shader_program: None,
            assets: Assets::new()
        }
    }

    // TODO: Breadcrumb - refactor this method
    pub fn start(&mut self) {
        clog!("Starting!");

        self.assets.load_mesh(&self.current_model);

        let canvas_id = "mesh-visualizer";

        let canvas = document.create_canvas_element("canvas");
        canvas.set_width(500);
        canvas.set_height(500);
        canvas.set_id(canvas_id);
        document.body().append_canvas_child(canvas);

        let canvas = document.get_canvas_element_by_id(canvas_id);
        let gl = canvas.get_context("webgl");

        let gl_depth_test = 2929;
        gl.enable(gl_depth_test);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        // gl.FRAGMENT_SHADER
        let fragment_shader_type = 35632;
        // gl.VERTEX_SHADER
        let vertex_shader_type = 35633;

        let frag_shader = gl.create_shader(fragment_shader_type);
        let vert_shader = gl.create_shader(vertex_shader_type);

        gl.shader_source(
            &vert_shader,
            include_str!("./non-skinned-vertex-shader.glsl"),
        );
        gl.shader_source(
            &frag_shader,
            include_str!("./non-skinned-fragment-shader.glsl"),
        );

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
        gl.use_program(&shader_program);

        gl.viewport(0, 0, 500, 500);

        self.gl = Some(gl);
        self.non_skinned_shader_program = Some(shader_program);
    }

    pub fn draw(&self) {
        if self.gl.is_none() {
            return;
        }

        let mesh = self.assets.meshes();
        let mesh = mesh.borrow();
        let mesh = mesh.get(&self.current_model);

        if mesh.is_none() {
            return;
        }

        let mesh = mesh.unwrap();

        let gl = self.gl.as_ref().unwrap();

        let shader_program = self.non_skinned_shader_program.as_ref().unwrap();

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
            Point3::new(1.0, 2.0, -2.0),
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
        gl.buffer_f32_data(gl_array_buffer, mesh.vertex_positions.clone(), static_draw);
        gl.vertex_attrib_pointer(vert_pos_attrib, 3, gl_FLOAT, false, 0, 0);

        let vert_normal_buffer = gl.create_buffer();
        gl.bind_buffer(gl_array_buffer, &vert_normal_buffer);
        // TODO: Remove clone
        gl.buffer_f32_data(gl_array_buffer, mesh.vertex_normals.clone(), static_draw);
        gl.vertex_attrib_pointer(vert_normal_attrib, 3, gl_FLOAT, false, 0, 0);

        let index_buffer = gl.create_buffer();
        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        // TODO: Remove clone
        gl.buffer_u16_data(
            gl_ELEMENT_ARRAY_BUFFER,
            mesh.vertex_position_indices.clone(),
            static_draw,
        );

        let gl_TRIANGLES = 4;
        let gl_UNSIGNED_SHORT = 5123;

        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        gl.draw_elements(
            gl_TRIANGLES,
            mesh.vertex_position_indices.len() as u16,
            gl_UNSIGNED_SHORT,
            0,
        );

        // TODO: Breadcrumb - plan and implement unit testing a skinned mesh export

        // TODO: Pass uCameraPos into fragment shader

        // TODO: Plan and implement a textured cube test
        // `textured_cube.{rs,blend}`. create an `img` element and add a source, then use
        // that image as a texture via hard coded uv coordinates.

        // TODO: Split this method up / clean up the var names

        // TODO: Render a cube instead of a triangle

        // TODO: Add camera controls
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

#[cfg(test)]
mod tests {
    #[test]
    fn foo() {}
}
