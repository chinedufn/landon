use virtual_dom_rs::prelude::*;

pub struct CanvasView {}

impl View for CanvasView {
    fn render(&self) -> VirtualNode {
        let canvas = html! {
          <canvas id="mesh-visualizer" width="500" height="500"></canvas>
        };

        canvas
    }
}
