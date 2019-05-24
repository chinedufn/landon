//! virtual-dom-rs views for the user interface

use crate::state_wrapper::StateWrapper;
use crate::view::canvas::CanvasView;
use crate::view::controls::ControlsView;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;
use virtual_dom_rs::View;

pub struct MainView {
    pub wrapper: Rc<RefCell<StateWrapper>>,
}

mod canvas;
mod controls;

impl View for MainView {
    fn render(&self) -> VirtualNode {
        let canvas = CanvasView {
            wrapper: &self.wrapper,
        };

        let view = html! {
          // Contains two side by side divs, one for the canvas, one for the controls
          // (sliders, buttons, etc)
          <div style="display: flex;">
             <div>
              {canvas.render()}
             </div>
             {ControlsView {}.render()}

           </div>
        };

        view
    }
}
