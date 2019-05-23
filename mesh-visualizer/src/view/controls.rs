use virtual_dom_rs::prelude::*;

pub struct ControlsView {}

impl View for ControlsView {
    fn render(&self) -> VirtualNode {
        let controls = html! {
          <div>
          Controls go here!!
          </div>
        };

        controls
    }
}
