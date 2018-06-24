#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate blender_mesh;
extern crate wasm_bindgen;
extern crate cgmath;

use blender_mesh::BlenderMesh;

use wasm_bindgen::prelude::*;

pub mod web_apis;

use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;
use web_apis::*;
use std::cell::RefCell;

use cgmath::Matrix4;
use cgmath::Vector3;
use cgmath::Point3;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn download_model(model_name: &str, cb: &Closure<FnMut(String)>);
}

#[wasm_bindgen]
pub struct App {
    /// The model that the user is currently viewing in their browser
    current_model: Rc<RefCell<Option<String>>>,
    /// All of the models that we have downloaded and can render
    meshes: Rc<RefCell<HashMap<String, BlenderMesh>>>,
    /// A handle into the WebGL context for our canvas
    gl: Option<WebGLRenderingContext>,
    non_skinned_shader_program: Option<WebGLProgram>,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        App {
            meshes: Rc::new(RefCell::new(HashMap::new())),
            current_model: Rc::new(RefCell::new(None)),
            gl: None,
            non_skinned_shader_program: None,
        }
    }

    pub fn start(&mut self) {
        clog!("Starting!");


        let current_model_clone = Rc::clone(&self.current_model);
        let meshes_clone = Rc::clone(&self.meshes);

        let save_model_in_state = move |model_json: String| {
            let mut mesh = BlenderMesh::from_json(&model_json).unwrap();

            mesh.combine_vertex_indices();
            mesh.triangulate();

            meshes_clone.borrow_mut().insert("dist/cube.json".to_string(), mesh);
            *current_model_clone.borrow_mut() = Some("dist/cube.json".to_string());
        };

        let on_model_load = Closure::new(save_model_in_state);

        download_model("dist/cube.json", &on_model_load);

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

        // gl.FRAGMENT_SHAADER
        let fragment_shader_type = 35632;
        // gl.VERTEX_SHAADER
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
            clog!("Vertex shader compilation errors: {}", frag_log);
        }

        let shader_program = gl.create_program();
        gl.attach_shader(&shader_program, &frag_shader);
        gl.attach_shader(&shader_program, &vert_shader);
        gl.link_program(&shader_program);
        gl.use_program(&shader_program);

        gl.viewport(0, 0, 500, 500);

        self.gl = Some(gl);
        self.non_skinned_shader_program = Some(shader_program);

        on_model_load.forget();
    }

    pub fn draw(&self) {
        if self.gl.is_none() {
            return;
        }

        let current_model = self.current_model.borrow();
        let current_model = current_model.as_ref();
        if current_model.is_none() {
            return;
        }
        let current_model = current_model.unwrap();

        let mesh = self.meshes.borrow();
        let mesh = mesh.get(current_model).unwrap();

        let gl = self.gl.as_ref().unwrap();

        let shader_program = self.non_skinned_shader_program.as_ref().unwrap();

        let vert_pos_attrib = gl.get_attrib_location(&shader_program, "aVertPos");
        gl.enable_vertex_attrib_array(vert_pos_attrib);

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

        // TODO: Breadcrumb - create a view matrix looking down on the model. multiple the view
        // and model matrix and store as mv matrix;
        let mut mv_matrix = Matrix4::look_at(Point3::new(1.0, 2.0, -10.0), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        // TODO: Multiply without new allocation
        mv_matrix = mv_matrix * model_matrix;

        let mv_matrix = vec_from_matrix4(&mv_matrix);

        let p_matrix_uni = gl.get_uniform_location(&shader_program, "uPMatrix");
        let mv_matrix_uni = gl.get_uniform_location(&shader_program, "uMVMatrix");

        gl.uniform_matrix_4fv(p_matrix_uni, false, p_matrix);
        gl.uniform_matrix_4fv(mv_matrix_uni, false, mv_matrix);

        let array_buffer = 34962;
        let gl_ELEMENT_ARRAY_BUFFER = 34963;

        let static_draw = 35044;

        let vert_pos_buffer = gl.create_buffer();
        gl.bind_buffer(array_buffer, &vert_pos_buffer);
        // TODO: Remove clone
        gl.buffer_f32_data(array_buffer, mesh.vertex_positions.clone(), static_draw);

        let gl_FLOAT = 5126;

        gl.vertex_attrib_pointer(vert_pos_attrib, 3, gl_FLOAT, false, 0, 0);

        let index_buffer = gl.create_buffer();
        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        // TODO: Remove clone
        gl.buffer_u16_data(gl_ELEMENT_ARRAY_BUFFER, mesh.vertex_position_indices.clone(), static_draw);

        let gl_TRIANGLES = 4;
        let gl_UNSIGNED_SHORT = 5123;

        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        gl.draw_elements(gl_TRIANGLES, mesh.vertex_position_indices.len() as u16, gl_UNSIGNED_SHORT, 0);

        // TODO: Add normals and lighting to non-skinned shader

        // TODO: Make look at camera looking down at mesh (move math into separate module)

        // TODO: `textured_cube.{rs,blend}`. create an `img` element and add a source, then use
        // that image as a texture via hard coded uv coordinates.

        // TODO: Split this method up / clean up the var names

        // TODO: Render a cube instead of a triangle

        // TODO: Add camera controls
    }
}

fn vec_from_matrix4 (mat4: &Matrix4<f32>) -> Vec<f32> {
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
    fn it_works() {}
}
