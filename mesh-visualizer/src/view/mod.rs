//! virtual-dom-rs views for the user interface

use crate::view::canvas::CanvasView;
use crate::view::controls::ControlsView;
use virtual_dom_rs::prelude::*;
use virtual_dom_rs::View;

pub struct MainView {}

mod canvas;
mod controls;

impl View for MainView {
    fn render(&self) -> VirtualNode {
        let view = html! {
          // Contains two side by side divs, one for the canvas, one for the controls
          // (sliders, buttons, etc)
          <div style="display: flex;">
             <div>
               {CanvasView{}.render()}
             </div>
             {ControlsView {}.render()}

           </div>
        };

        view
    }
}
