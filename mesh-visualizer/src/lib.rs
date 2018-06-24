#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate blender_mesh;
extern crate wasm_bindgen;

use blender_mesh::BlenderMesh;

use wasm_bindgen::prelude::*;

pub mod web_apis;
use std::collections::HashMap;
use std::f32::consts::PI;
use web_apis::*;

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
    meshes: HashMap<String, BlenderMesh>,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        App { meshes: vec![] }
    }

    pub fn start(&self) {
        clog!("Starting!");

        let save_model_in_state = move |model_json: String| {
            let mesh = BlenderMesh::from_json(&model_json).unwrap();
        };

        let on_model_load = Closure::new(save_model_in_state);

        download_model("dist/cube.json", &on_model_load);

        let canvas_id = "mesh-visualizer";

        // Temporarily using u16's until I can get GLbitfield / Glenum etc working
        let color_buffer_bit = 16384;
        let depth_buffer_bit = 256;
        // color_buffer_bit | depth_buffer_bit
        let bitfield = 16640;
        let depth_test = 2929;

        let canvas = document.create_canvas_element("canvas");
        canvas.set_width(500);
        canvas.set_height(500);
        canvas.set_id(canvas_id);
        document.body().append_canvas_child(canvas);

        let canvas = document.get_canvas_element_by_id(canvas_id);
        let gl = canvas.get_context("webgl");

        gl.enable(depth_test);
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

        let vert_pos_attrib = gl.get_attrib_location(&shader_program, "aVertPos");
        gl.enable_vertex_attrib_array(vert_pos_attrib);

        let p_matrix_uni = gl.get_uniform_location(&shader_program, "uPMatrix");
        let mv_matrix_uni = gl.get_uniform_location(&shader_program, "uMVMatrix");

        gl.viewport(0, 0, 500, 500);
        gl.clear(bitfield);

        let p_matrix = perspective(PI / 3.0, 1.0, 0.1, 100.0);
        let mv_matrix = vec![
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -5.0, 1.0,
        ];

        gl.uniform_matrix_4fv(p_matrix_uni, false, p_matrix);
        gl.uniform_matrix_4fv(mv_matrix_uni, false, mv_matrix);

        let array_buffer = 34962;
        let gl_ELEMENT_ARRAY_BUFFER = 34963;

        let static_draw = 35044;

        let vert_pos_buffer = gl.create_buffer();
        gl.bind_buffer(array_buffer, &vert_pos_buffer);
        let vertices = vec![1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        gl.buffer_f32_data(array_buffer, vertices, static_draw);

        let gl_FLOAT = 5126;

        gl.vertex_attrib_pointer(vert_pos_attrib, 3, gl_FLOAT, false, 0, 0);

        let index_buffer = gl.create_buffer();
        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);
        let pos_indices = vec![0, 1, 2];
        gl.buffer_u16_data(gl_ELEMENT_ARRAY_BUFFER, pos_indices, static_draw);

        let gl_TRIANGLES = 4;
        let gl_UNSIGNED_SHORT = 5123;

        gl.bind_buffer(gl_ELEMENT_ARRAY_BUFFER, &index_buffer);

        gl.draw_elements(gl_TRIANGLES, 3, gl_UNSIGNED_SHORT, 0);

        // TODO: Add normals and lighting to non-skinned shader

        // TODO: Make look at camera looking down at mesh (move math into separate module)

        // TODO: `textured_cube.{rs,blend}`. create an `img` element and add a source, then use
        // that image as a texture via hard coded uv coordinates.

        // TODO: Split this method up / clean up the var names

        // TODO: Render a cube instead of a triangle

        // TODO: Add camera controls

        on_model_load.forget();
    }

    pub fn draw(&self) {
        clog!("Draw!");
    }
}

// Ported from https://github.com/stackgl/gl-mat4/blob/master/perspective.js
fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Vec<f32> {
    let f = 1.0 / (fovy / 2.0).tan();

    let nf = 1.0 / (near - far);

    vec![
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) * nf,
        -1.0,
        0.0,
        0.0,
        (2.0 * far * near) * nf,
        0.0,
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
