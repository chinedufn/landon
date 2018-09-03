#![feature(use_extern_macros)]

extern crate blender_armature;
extern crate blender_mesh;
extern crate cgmath;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[macro_use]
pub mod web_apis;
use web_apis::*;

use std::rc::Rc;

mod assets;
use assets::Assets;
use render::Renderer;
use shader::ShaderSystem;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_use]
mod shader;

mod render;

static GL_DEPTH_TEST: u16 = 2929;

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn download_mesh(mesh_name: &str, mesh_url: &str, cb: &Closure<FnMut(String, String)>);
}

mod state;
use state::State;
use std::cell::RefCell;

#[wasm_bindgen]
pub struct App {
    /// The model that the user is currently viewing in their browser
    current_model: String,
    /// A handle into the WebGL context for our canvas
    state: Rc<State>,
    assets: Rc<RefCell<Assets>>,
    renderer: Renderer,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        let canvas = App::create_canvas();
        document.body().append_canvas_child(&canvas);

        let gl = Rc::new(App::create_webgl_context(&canvas));

        let shader_sys = ShaderSystem::new(Rc::clone(&gl));

        let state = Rc::new(State::new());

        let mut assets = Rc::new(RefCell::new(Assets::new()));

        let renderer = Renderer::new(
            Rc::clone(&gl),
            Rc::clone(&assets),
            shader_sys,
            Rc::clone(&state),
        );

        App {
            current_model: "TexturedCube".to_string(),
            state: Rc::clone(&state),
            assets: Rc::clone(&assets),
            renderer,
        }
    }

    pub fn start(&mut self) {
        self.assets.borrow_mut().load_mesh(&self.current_model);
        self.assets.borrow_mut().load_armature("LetterFArmature");
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
