//! TODO: Use percy to render a UI that lets you select models to render
//! and the animation to play

#![feature(use_extern_macros)]

extern crate blender_armature;
extern crate blender_mesh;
extern crate cgmath;
extern crate wasm_bindgen;
extern crate serde;
extern crate serde_json;

use wasm_bindgen::prelude::*;

#[macro_use]
pub mod web_apis;
use crate::web_apis::*;

use std::rc::Rc;

mod assets;
use crate::assets::Assets;
use crate::render::Renderer;
use crate::shader::ShaderSystem;

static GL_TEXTURE_2D: u16 = 3553;
static TEXTURE_UNIT_0: u16 = 33984;
static UNPACK_FLIP_Y_WEBGL: u16 = 37440;
static GL_NEAREST: u16 = 9728;
static GL_LINEAR: u16 = 9729;
static TEXTURE_MIN_FILTER: u16 = 10241;
static TEXTURE_MAG_FILTER: u16 = 10240;
static GL_RGBA: u16 = 6408;
static GL_UNSIGNED_BYTE: u16 = 5121;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_use]
mod shader;

mod render;

static GL_DEPTH_TEST: u16 = 2929;

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn download_string(url: String, cb: &Closure<FnMut(String)>);
    fn download_texture(cb: &Closure<FnMut(HTMLImageElement)>);
}

mod state;
use crate::shader::ShaderType;
use crate::state::State;
use std::cell::RefCell;

#[wasm_bindgen]
pub struct App {
    gl: Rc<WebGLRenderingContext>,
    /// A handle into the WebGL context for our canvas
    state: Rc<State>,
    assets: Rc<RefCell<Assets>>,
    shader_sys: Rc<ShaderSystem>,
    renderer: Renderer,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        let canvas = App::create_canvas();
        document.body().append_canvas_child(&canvas);

        let gl = Rc::new(App::create_webgl_context(&canvas));

        let shader_sys = Rc::new(ShaderSystem::new(Rc::clone(&gl)));

        let state = Rc::new(State::new());

        let assets = Rc::new(RefCell::new(Assets::new()));

        let renderer = Renderer::new(
            Rc::clone(&gl),
            Rc::clone(&assets),
            Rc::clone(&shader_sys),
            Rc::clone(&state),
        );

        App {
            gl: Rc::clone(&gl),
            state: Rc::clone(&state),
            assets: Rc::clone(&assets),
            shader_sys,
            renderer,
        }
    }

    pub fn create_texture(&self) -> WebGLTexture {
        self.gl.create_texture()
    }

    pub fn start(&mut self) {
        self.assets
            .borrow_mut()
            .load_mesh(&self.state.current_model);
        //        self.assets.borrow_mut().load_armature("LetterFArmature");
    }

    pub fn set_texture(&mut self, image: HTMLImageElement) {
        let texture = self.gl.create_texture();

        self.gl.active_texture(TEXTURE_UNIT_0);

        // TODO: When we're done bind texture (null)
        self.gl.bind_texture(GL_TEXTURE_2D, texture);

        self.gl.pixel_store_i(UNPACK_FLIP_Y_WEBGL, true);

        self.gl.tex_parameter_i(GL_TEXTURE_2D, TEXTURE_MIN_FILTER, GL_LINEAR);
        self.gl.tex_parameter_i(GL_TEXTURE_2D, TEXTURE_MAG_FILTER, GL_LINEAR);


        self.gl.tex_image_2D(GL_TEXTURE_2D, 0, GL_RGBA, GL_RGBA, GL_UNSIGNED_BYTE, image);

        self.gl.uniform1i(
            self.gl.get_uniform_location(
                &self.shader_sys.get_shader(&ShaderType::NonSkinned).program,
                "uSampler",
            ),
            0,
        );
//        self.gl.uniform1i(
//            self.gl.get_uniform_location(
//                &self.shader_sys.get_shader(&ShaderType::NonSkinned).program,
//                "uUseTexture",
//            ),
//            1
//        );
    }

    pub fn draw(&self) {
        self.renderer.render(&self.state);
        // TODO: Pass uCameraPos into fragment shader

        // TODO: Plan and implement a textured cube test
        // `textured_cube.{rs,blend}`. create an `img` element and add a source, then use
        // that image as a texture via hard coded uv coordinates.

        // TODO: Split this method up / clean up the var names

        // TODO: Render a cube instead of a triangle

        // TODO: Add camera controls
    }

    fn create_canvas() -> HTMLCanvasElement {
        let canvas_id = "mesh-visualizer";

        let canvas = document.create_canvas_element("canvas");
        canvas.set_width(500);
        canvas.set_height(500);
        canvas.set_id(canvas_id);

        canvas
    }

    fn create_webgl_context(canvas: &HTMLCanvasElement) -> WebGLRenderingContext {
        let gl = canvas.get_context("webgl");

        gl.enable(GL_DEPTH_TEST);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.viewport(0, 0, 500, 500);

        gl
    }
}
