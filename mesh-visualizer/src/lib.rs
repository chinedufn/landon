//! TODO: Use percy to render a UI that lets you select models to render
//! and the animation to play

use crate::assets::Assets;
use crate::render::Renderer;
use crate::shader::ShaderSystem;
use crate::shader::ShaderType;
use crate::state::State;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;
mod assets;
mod render;
mod shader;
mod state;

#[wasm_bindgen]
pub struct App {
    gl: Rc<WebGlRenderingContext>,
    /// A handle into the WebGL context for our canvas
    state: Rc<State>,
    assets: Rc<RefCell<Assets>>,
    shader_sys: Rc<ShaderSystem>,
    renderer: Renderer,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> App {
        console_error_panic_hook::set_once();

        let canvas = App::create_canvas().unwrap();
        let document = window().unwrap().document().unwrap();
        document.body().unwrap().append_child(&canvas);

        let gl = Rc::new(App::create_webgl_context(&canvas).unwrap());

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

    pub fn start(&mut self) {
        self.assets
            .borrow_mut()
            .load_meshes();
        //        self.assets.borrow_mut().load_armature("LetterFArmature");
    }

    pub fn set_texture(&mut self, image: HtmlImageElement) {
        let texture = self.gl.create_texture();

        self.gl.active_texture(GL::TEXTURE0);

        self.gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());

        self.gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);

        self.gl
            .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        self.gl
            .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        self.gl.tex_image_2d_with_u32_and_u32_and_image(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            &image,
        );

        self.gl.uniform1i(
            self.gl
                .get_uniform_location(
                    self
                        .shader_sys
                        .get_shader(&ShaderType::NonSkinned)
                        .as_ref()
                        .unwrap()
                        .program
                        .as_ref()
                        .unwrap(),
                    "uSampler",
                )
                .as_ref(),
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

    fn create_canvas() -> Result<HtmlCanvasElement, JsValue> {
        let canvas_id = "mesh-visualizer";

        let window = window().unwrap();
        let document = window.document().unwrap();

        let canvas: HtmlCanvasElement = document.create_element("canvas").unwrap().dyn_into()?;

        canvas.set_width(500);
        canvas.set_height(500);
        canvas.set_id(canvas_id);

        Ok(canvas)
    }

    fn create_webgl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, JsValue> {
        let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

        gl.enable(GL::DEPTH_TEST);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.viewport(0, 0, 500, 500);

        Ok(gl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {}
}
