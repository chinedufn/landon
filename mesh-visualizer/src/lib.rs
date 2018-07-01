#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate blender_armature;
extern crate blender_mesh;
extern crate cgmath;
extern crate wasm_bindgen;

use blender_mesh::BlenderMesh;

use wasm_bindgen::prelude::*;

#[macro_use]
pub mod web_apis;
use web_apis::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;

mod assets;
use assets::Assets;
use render::Renderer;
use shader::ShaderSystem;
use shader::ShaderType;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_use]
mod shader;

mod render;

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn download_mesh(mesh_name: &str, mesh_url: &str, cb: &Closure<FnMut(String, String)>);
}

// TODO: state.rs module
pub struct State {}
impl State {
    fn new() -> State {
        State {}
    }
}

#[wasm_bindgen]
pub struct App {
    /// The model that the user is currently viewing in their browser
    current_model: String,
    /// A handle into the WebGL context for our canvas
    gl: Option<Rc<WebGLRenderingContext>>,
    state: Rc<State>,
    renderer: Option<Renderer>,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        App {
            current_model: "LetterF".to_string(),
            gl: None,
            state: Rc::new(State::new()),
            renderer: None,
        }
    }

    // TODO: Breadcrumb - refactor this method
    pub fn start(&mut self) {
        clog!("Starting!");

        let mut assets = Assets::new();
        assets.load_mesh(&self.current_model);

        let canvas_id = "mesh-visualizer";

        let canvas = document.create_canvas_element("canvas");
        canvas.set_width(500);
        canvas.set_height(500);
        canvas.set_id(canvas_id);
        document.body().append_canvas_child(canvas);

        let canvas = document.get_canvas_element_by_id(canvas_id);
        // TODO: Create context in our new function so that we don't need to have Option<gl>
        // and Option<Renderer>
        let gl = canvas.get_context("webgl");

        let gl_depth_test = 2929;
        gl.enable(gl_depth_test);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        let gl = Rc::new(gl);
        let mut shader_sys = ShaderSystem::new(Rc::clone(&gl));

        let renderer = Renderer::new(Rc::clone(&gl), assets, shader_sys, Rc::clone(&self.state));

        gl.viewport(0, 0, 500, 500);

        self.gl = Some(gl);
        self.renderer = Some(renderer);
    }

    pub fn draw(&self) {
        if self.gl.is_none() {
            return;
        }

        self.renderer.as_ref().unwrap().render();
        // TODO breadcrumb - create self.skinned_shader_program and store it in our struct.. then
        // send down an armature.. store it in our assets.rs and use the inverse bind poses to
        // render our model (inverse the inverse bind poses to get the bind poses and pass those
        // into our renderer as bones).

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

#[cfg(test)]
mod tests {
    #[test]
    fn foo() {}
}
