//! Need to re-write this crate. Wrote it when I was a Rust noob and it's beyond repair

// #![feature(proc_macro_hygiene)]

// #[macro_use]
// extern crate log;

// #[macro_use]
// extern crate virtual_dom_rs;

// use virtual_dom_rs::prelude::*;

// use crate::assets::Assets;
// use crate::render::Renderer;
// use crate::shader::ShaderSystem;
// use crate::state_wrapper::{State, StateWrapper};
// use crate::view::MainView;
// use state_wrapper::Msg;
// use std::cell::RefCell;
// use std::rc::Rc;
// use virtual_dom_rs::DomUpdater;
// use wasm_bindgen;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;
// use web_sys::WebGlRenderingContext as GL;
// use web_sys::*;

// mod assets;
// mod render;
// mod shader;
// mod state_wrapper;
// mod view;

// #[wasm_bindgen]
// pub struct App {
//     gl: Rc<WebGlRenderingContext>,
//     /// A handle into the WebGL context for our canvas
//     state_wrap: Rc<RefCell<StateWrapper>>,
//     assets: Rc<RefCell<Assets>>,
//     shader_sys: Rc<ShaderSystem>,
//     renderer: Renderer,
//     dom_updater: DomUpdater,
// }

// #[wasm_bindgen]
// impl App {
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> App {
//         console_log::init();
//         console_error_panic_hook::set_once();

//         let window = web_sys::window().unwrap();
//         let document = window.document().unwrap();
//         let body = document.body().unwrap();

//         let assets = Rc::new(RefCell::new(Assets::new()));

//         let state_wrap = Rc::new(RefCell::new(StateWrapper::new(
//             State::new(),
//             Rc::clone(&assets),
//         )));

//         let view = MainView {
//             wrapper: Rc::clone(&state_wrap),
//         }
//         .render();

//         let dom_updater = DomUpdater::new_append_to_mount(view, &body);

//         let canvas = document
//             .get_element_by_id("mesh-visualizer")
//             .unwrap()
//             .dyn_into()
//             .unwrap();

//         let gl = Rc::new(App::create_webgl_context(&canvas).unwrap());

//         let shader_sys = Rc::new(ShaderSystem::new(Rc::clone(&gl)));

//         let renderer = Renderer::new(Rc::clone(&gl), Rc::clone(&assets), Rc::clone(&shader_sys));

//         App {
//             gl: Rc::clone(&gl),
//             state_wrap: Rc::clone(&state_wrap),
//             assets: Rc::clone(&assets),
//             shader_sys,
//             renderer,
//             dom_updater,
//         }
//     }

//     pub fn start(&mut self) {
//         self.assets
//             .borrow_mut()
//             .load_meshes(Rc::clone(&self.state_wrap));
//         //        self.assets.borrow_mut().load_armature("LetterFArmature");
//     }

//     pub fn set_texture(&mut self, image: HtmlImageElement) {
//         let texture = self.gl.create_texture();

//         self.gl.active_texture(GL::TEXTURE0);

//         self.gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());

//         self.gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);

//         self.gl
//             .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
//         self.gl
//             .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

//         self.gl
//             .tex_image_2d_with_u32_and_u32_and_image(
//                 GL::TEXTURE_2D,
//                 0,
//                 GL::RGBA as i32,
//                 GL::RGBA,
//                 GL::UNSIGNED_BYTE,
//                 &image,
//             )
//             .unwrap();
//     }

//     pub fn draw(&self) {
//         self.renderer.render(&self.state_wrap.borrow());
//         // TODO: Pass uCameraPos into fragment shader

//         // TODO: Plan and implement a textured cube test
//         // `textured_cube.{rs,blend}`. create an `img` element and add a source, then use
//         // that image as a texture via hard coded uv coordinates.

//         // TODO: Split this method up / clean up the var names

//         // TODO: Render a cube instead of a triangle

//         // TODO: Add camera controls
//     }

//     fn create_webgl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, JsValue> {
//         let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

//         gl.enable(GL::DEPTH_TEST);
//         gl.clear_color(0.0, 0.0, 0.0, 1.0);
//         gl.viewport(0, 0, 500, 500);

//         Ok(gl)
//     }
// }
