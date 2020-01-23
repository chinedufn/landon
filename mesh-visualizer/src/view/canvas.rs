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
        let _wrapper = Rc::clone(&self.wrapper);

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

    attach_mouse_down_handler(&canvas, Rc::clone(&wrapper))?;
    attach_mouse_up_handler(&canvas, Rc::clone(&wrapper))?;
    attach_mouse_move_handler(&canvas, Rc::clone(&wrapper))?;
    attach_mouse_wheel_handler(&canvas, Rc::clone(&wrapper))?;

    attach_touch_start_handler(&canvas, Rc::clone(&wrapper))?;
    attach_touch_move_handler(&canvas, Rc::clone(&wrapper))?;
    attach_touch_end_handler(&canvas, Rc::clone(&wrapper))?;

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
    let on_mouse_wheel = Closure::wrap(Box::new(on_mouse_wheel) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("wheel", on_mouse_wheel.as_ref().unchecked_ref())?;
    on_mouse_wheel.forget();

    Ok(())
}

fn attach_mouse_down_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        let x = event.client_x();
        let y = event.client_y();
        wrapper.borrow_mut().msg(Msg::MouseDown(x, y));
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;

    handler.forget();

    Ok(())
}

fn attach_mouse_up_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |_event: web_sys::MouseEvent| {
        wrapper.borrow_mut().msg(Msg::MouseUp);
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_mouse_move_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let x = event.client_x();
        let y = event.client_y();
        wrapper.borrow_mut().msg(Msg::MouseMove(x, y));
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_mouse_wheel_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |event: web_sys::WheelEvent| {
        event.prevent_default();

        let zoom_amount = event.delta_y() / 50.;

        wrapper.borrow_mut().msg(Msg::Zoom(zoom_amount as f32));
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_touch_start_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |event: web_sys::TouchEvent| {
        let touch = event.touches().item(0).expect("First Touch");
        let x = touch.client_x();
        let y = touch.client_y();
        wrapper.borrow_mut().msg(Msg::MouseDown(x, y));
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("touchstart", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_touch_move_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |event: web_sys::TouchEvent| {
        event.prevent_default();
        let touch = event.touches().item(0).expect("First Touch");
        let x = touch.client_x();
        let y = touch.client_y();
        wrapper.borrow_mut().msg(Msg::MouseMove(x, y));
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("touchmove", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}

fn attach_touch_end_handler(
    canvas: &web_sys::Element,
    wrapper: Rc<RefCell<StateWrapper>>,
) -> Result<(), JsValue> {
    let handler = move |_event: web_sys::TouchEvent| {
        wrapper.borrow_mut().msg(Msg::MouseUp);
    };

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("touchend", handler.as_ref().unchecked_ref())?;
    handler.forget();

    Ok(())
}
