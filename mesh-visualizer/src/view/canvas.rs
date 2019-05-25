use crate::state_wrapper::StateWrapper;
use crate::Msg;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub struct CanvasView<'a> {
    pub wrapper: &'a Rc<RefCell<StateWrapper>>,
}

impl<'a> View for CanvasView<'a> {
    fn render(&self) -> VirtualNode {
        let wrapper = Rc::clone(&self.wrapper);

        let canvas = html! {
          <canvas
           id="mesh-visualizer"
           width="500"
           height="500"
           on_create_elem=move |elem: web_sys::Element| {
             attach_listeners(elem, &wrapper);
           }
          >
          </canvas>
        };

        canvas
    }
}

fn attach_listeners(
    canvas: web_sys::Element,
    wrapper: &Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    listen_for_zoom(&canvas, Rc::clone(&wrapper))?;

    Ok(())
}

fn listen_for_zoom(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let on_mouse_wheel = move |event: web_sys::WheelEvent| {
        event.prevent_default();

        let zoom_amount = event.delta_y() / 50.;

        wrapper.borrow_mut().msg(Msg::Zoom(zoom_amount as f32));
    };
    let on_mouse_wheel = Closure::wrap(Box::new(on_mouse_wheel) as Box<FnMut(_)>);

    canvas.add_event_listener_with_callback("wheel", on_mouse_wheel.as_ref().unchecked_ref())?;
    on_mouse_wheel.forget();

    Ok(())
}
