#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

pub mod web_apis;
use web_apis::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[wasm_bindgen]
pub struct App {
}

#[wasm_bindgen]
impl App {
    pub fn new () -> App {
        App {}
    }

    pub fn start () {
        clog!("Starting!");

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

        gl.shader_source(&vert_shader, include_str!("./non-skinned-vertex-shader.glsl"));
        gl.shader_source(&frag_shader, include_str!("./non-skinned-fragment-shader.glsl"));

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

        let p_matrix = vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let mv_matrix = vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

        gl.uniform_matrix_4fv(p_matrix_uni, false, p_matrix);
        gl.uniform_matrix_4fv(mv_matrix_uni, false, mv_matrix);

        let array_buffer = 34962;
        let element_array_buffer = 34963;

        let static_draw = 35044;

//        var vertexPositionAttribute = gl.getAttribLocation(shaderProgram, 'aVertexPosition')
//        gl.enableVertexAttribArray(vertexPositionAttribute)
    }
}
