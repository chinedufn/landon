#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    type HTMLDocument;
    static document: HTMLDocument;
    #[wasm_bindgen(method)]
    fn createElement(this: &HTMLDocument, tagName: &str) -> Element;
    #[wasm_bindgen(method, js_name = createElement)]
    fn create_canvas_element(this: &HTMLDocument, tagName: &str) -> HTMLCanvasElement;
    #[wasm_bindgen(method, getter)]
    fn body(this: &HTMLDocument) -> Element;

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

        let canvas = document.create_canvas_element("canvas");
        canvas.set_width(500);
        canvas.set_height(500);
        document.body().append_canvas_child(canvas);
    }
}
