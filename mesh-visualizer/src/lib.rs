#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    type HTMLDocument;

    static document: HTMLDocument;

    #[wasm_bindgen(method, js_name = createElement)]
    fn create_element(this: &HTMLDocument, tagName: &str) -> Element;

    #[wasm_bindgen(method, js_name = createElement)]
    fn create_canvas_element(this: &HTMLDocument, tagName: &str) -> HTMLCanvasElement;

    #[wasm_bindgen(method, getter)]
    fn body(this: &HTMLDocument) -> Element;

    #[wasm_bindgen(method, js_name = getElementById)]
    fn get_canvas_element_by_id(this: &HTMLDocument, id: &str) -> HTMLCanvasElement;

    type Element;

    #[wasm_bindgen(method, setter = innerHTML)]
    fn set_inner_html(this: &Element, html: &str);

    #[wasm_bindgen(method, js_name = appendChild)]
    fn append_child(this: &Element, other: Element);

    #[wasm_bindgen(method, js_name = appendChild)]
    fn append_canvas_child(this: &Element, other: HTMLCanvasElement);

    type HTMLCanvasElement;

    #[wasm_bindgen(method, setter = width)]
    fn set_width(this: &HTMLCanvasElement, width: u16);

    #[wasm_bindgen(method, setter = height)]
    fn set_height(this: &HTMLCanvasElement, height: u16);

    #[wasm_bindgen(method, setter = id)]
    fn set_id(this: &HTMLCanvasElement, id: &str);

    #[wasm_bindgen(method, js_name = getContext)]
    fn get_context(this: &HTMLCanvasElement, context_type: &str) -> WebGLRenderingContext;

    type WebGLRenderingContext;

    #[wasm_bindgen(method)]
    fn enable(this: &WebGLRenderingContext, capability: u16);

    #[wasm_bindgen(method, js_name = clearColor)]
    fn clear_color(this: &WebGLRenderingContext, r: f32, g: f32, b: f32, a: f32);

    #[wasm_bindgen(method)]
    fn clear(this: &WebGLRenderingContext, mask: u16);

    #[wasm_bindgen(method, js_name = createShader)]
    fn create_shader(this: &WebGLRenderingContext, shader_type: u16) -> WebGLShader;

    #[wasm_bindgen(method, js_name = shaderSource)]
    fn shader_source(this: &WebGLRenderingContext, shader: &WebGLShader, source: &str);

    #[wasm_bindgen(method, js_name = compileShader)]
    fn compile_shader(this: &WebGLRenderingContext, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = getShaderInfoLog)]
    fn get_shader_info_log(this: &WebGLRenderingContext, shader: &WebGLShader) -> String;

    // TODO: Figure out why these accessors are throwing errors. Create a repo to reproduce the
    // error and open an issue in wasm-bindgen repo
//    #[wasm_bindgen(method, getter)]
//    fn COLOR_BUFFER_BIT(this: &WebGLRenderingContext) -> GLbitfield;
//
//    #[wasm_bindgen(method, getter)]
//    fn DEPTH_BUFFER_BIT(this: &WebGLRenderingContext) -> GLbitfield;
//
//    #[wasm_bindgen(method, getter)]
//    fn DEPTH_TEST(this: &WebGLRenderingContext) -> GLenum;
//
//    #[wasm_bindgen(method, getter, js_name = FRAGMENT_SHADER)]
//    fn FRAGMENT_SHADER(this: &WebGLRenderingContext) -> u16;

    type GLenum;
    type GLbitfield;
    type WebGLShader;
}

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
        gl.clear(bitfield);

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
    }
}
