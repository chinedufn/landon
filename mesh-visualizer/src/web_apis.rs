use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type HTMLDocument;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);

    pub static document: HTMLDocument;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_element(this: &HTMLDocument, tagName: &str) -> Element;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_canvas_element(this: &HTMLDocument, tagName: &str) -> HTMLCanvasElement;

    #[wasm_bindgen(method, getter)]
    pub fn body(this: &HTMLDocument) -> Element;

    #[wasm_bindgen(method, js_name = getElementById)]
    pub fn get_canvas_element_by_id(this: &HTMLDocument, id: &str) -> HTMLCanvasElement;
}

#[wasm_bindgen]
extern "C" {
    pub type Element;

    #[wasm_bindgen(method, setter = innerHTML)]
    pub fn set_inner_html(this: &Element, html: &str);

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_child(this: &Element, other: Element);

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_canvas_child(this: &Element, other: HTMLCanvasElement);

    pub type HTMLCanvasElement;

    #[wasm_bindgen(method, setter = width)]
    pub fn set_width(this: &HTMLCanvasElement, width: u16);

    #[wasm_bindgen(method, setter = height)]
    pub fn set_height(this: &HTMLCanvasElement, height: u16);

    #[wasm_bindgen(method, setter = id)]
    pub fn set_id(this: &HTMLCanvasElement, id: &str);

    #[wasm_bindgen(method, js_name = getContext)]
    pub fn get_context(this: &HTMLCanvasElement, context_type: &str) -> WebGLRenderingContext;
}

#[wasm_bindgen]
extern "C" {
    pub type WebGLRenderingContext;

    #[wasm_bindgen(method)]
    pub fn enable(this: &WebGLRenderingContext, capability: u16);

    #[wasm_bindgen(method, js_name = clearColor)]
    pub fn clear_color(this: &WebGLRenderingContext, r: f32, g: f32, b: f32, a: f32);

    #[wasm_bindgen(method)]
    pub fn clear(this: &WebGLRenderingContext, mask: u16);

    #[wasm_bindgen(method, js_name = createShader)]
    pub fn create_shader(this: &WebGLRenderingContext, shader_type: u16) -> WebGLShader;

    #[wasm_bindgen(method, js_name = shaderSource)]
    pub fn shader_source(this: &WebGLRenderingContext, shader: &WebGLShader, source: &str);

    #[wasm_bindgen(method, js_name = compileShader)]
    pub fn compile_shader(this: &WebGLRenderingContext, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = getShaderInfoLog)]
    pub fn get_shader_info_log(this: &WebGLRenderingContext, shader: &WebGLShader) -> String;

    #[wasm_bindgen(method, js_name = attachShader)]
    pub fn attach_shader(
        this: &WebGLRenderingContext,
        program: &WebGLProgram,
        shader: &WebGLShader,
    );

    #[wasm_bindgen(method, js_name = createProgram)]
    pub fn create_program(this: &WebGLRenderingContext) -> WebGLProgram;

    #[wasm_bindgen(method, js_name = linkProgram)]
    pub fn link_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = useProgram)]
    pub fn use_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = getAttribLocation)]
    pub fn get_attrib_location(
        this: &WebGLRenderingContext,
        program: &WebGLProgram,
        attrib: &str,
    ) -> u16;

    #[wasm_bindgen(method, js_name = getUniformLocation)]
    pub fn get_uniform_location(
        this: &WebGLRenderingContext,
        program: &WebGLProgram,
        uniform: &str,
    ) -> WebGLUniformLocation;

    #[wasm_bindgen(method, js_name = enableVertexAttribArray)]
    pub fn enable_vertex_attrib_array(this: &WebGLRenderingContext, attribute: u16);

    #[wasm_bindgen(method)]
    pub fn viewport(this: &WebGLRenderingContext, x: u16, y: u16, width: u16, height: u16);

    #[wasm_bindgen(method, js_name = uniformMatrix4fv)]
    pub fn uniform_matrix_4fv(
        this: &WebGLRenderingContext,
        loc: WebGLUniformLocation,
        tranpose: bool,
        value: Vec<f32>,
    );

    #[wasm_bindgen(method, js_name = createBuffer)]
    pub fn create_buffer(this: &WebGLRenderingContext) -> WebGLBuffer;

    #[wasm_bindgen(method, js_name = bindBuffer)]
    pub fn bind_buffer(this: &WebGLRenderingContext, buffer_type: u16, buffer: &WebGLBuffer);

    #[wasm_bindgen(method, js_name = bufferData)]
    pub fn buffer_f32_data(
        this: &WebGLRenderingContext,
        buffer_type: u16,
        data: Vec<f32>,
        usage: u16,
    );

    #[wasm_bindgen(method, js_name = bufferData)]
    pub fn buffer_u16_data(
        this: &WebGLRenderingContext,
        buffer_type: u16,
        data: Vec<u16>,
        usage: u16,
    );

    #[wasm_bindgen(method, js_name = vertexAttribPointer)]
    pub fn vertex_attrib_pointer(
        this: &WebGLRenderingContext,
        index: u16,
        size: u8,
        kind: u16,
        normalized: bool,
        stride: u8,
        offset: u8,
    );

    #[wasm_bindgen(method, js_name = drawElements)]
    pub fn draw_elements(this: &WebGLRenderingContext, mode: u8, count: u16, kind: u16, offset: u8);

// TODO: Figure out why these accessors are throwing errors. Create a repo to reproduce the
// error and open an issue in wasm-bindgen repo
//    #[wasm_bindgen(method, getter)]
//    pub fn COLOR_BUFFER_BIT(this: &WebGLRenderingContext) -> GLbitfield;
//
//    #[wasm_bindgen(method, getter)]
//    pub fn DEPTH_BUFFER_BIT(this: &WebGLRenderingContext) -> GLbitfield;
//
//    #[wasm_bindgen(method, getter)]
//    pub fn DEPTH_TEST(this: &WebGLRenderingContext) -> GLenum;
//
//    #[wasm_bindgen(method, getter, js_name = FRAGMENT_SHADER)]
//    pub fn FRAGMENT_SHADER(this: &WebGLRenderingContext) -> u16;
}

#[wasm_bindgen]
extern "C" {
    pub type GLenum;
    pub type GLbitfield;
    pub type WebGLShader;
    pub type WebGLProgram;
    pub type WebGLUniformLocation;
    pub type WebGLBuffer;
}
