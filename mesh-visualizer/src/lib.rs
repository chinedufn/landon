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

    #[wasm_bindgen(method, js_name = attachShader)]
    fn attach_shader(this: &WebGLRenderingContext, program: &WebGLProgram, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = createProgram)]
    fn create_program(this: &WebGLRenderingContext) -> WebGLProgram;

    #[wasm_bindgen(method, js_name = linkProgram)]
    fn link_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = useProgram)]
    fn use_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = getAttribLocation)]
    fn get_attrib_location(this: &WebGLRenderingContext, program: &WebGLProgram, attrib: &str) -> u16;

    #[wasm_bindgen(method, js_name = getUniformLocation)]
    fn get_uniform_location(this: &WebGLRenderingContext, program: &WebGLProgram, uniform: &str) -> WebGLUniformLocation;

    #[wasm_bindgen(method, js_name = enableVertexAttribArray)]
    fn enable_vertex_attrib_array(this: &WebGLRenderingContext, attribute: u16);

    #[wasm_bindgen(method)]
    fn viewport(this: &WebGLRenderingContext, x: u16, y: u16, width: u16, height: u16);

    #[wasm_bindgen(method, js_name = uniformMatrix4fv)]
    fn uniform_matrix_4fv(this: &WebGLRenderingContext, loc: WebGLUniformLocation, tranpose: bool, value: Vec<f32>);

    #[wasm_bindgen(method, js_name = bindBuffer)]
    fn bind_buffer(this: &WebGLRenderingContext, buffer_type: u16, buffer: WebGLBuffer);

    #[wasm_bindgen(method, js_name = bufferData)]
    fn buffer_data(this: &WebGLRenderingContext, buffer_type: u16, data: Vec<f32>, usage: u16);

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
    type WebGLProgram;
    type WebGLUniformLocation;
    type WebGLBuffer;
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
