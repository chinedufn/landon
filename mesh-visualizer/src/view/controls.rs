use crate::state_wrapper::{Msg, State, StateWrapper};
use crate::virtual_dom_rs::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;

pub struct ControlsView<'a> {
    pub wrapper: &'a Rc<RefCell<StateWrapper>>,
}

impl<'a> View for ControlsView<'a> {
    fn render(&self) -> VirtualNode {
        let mesh_selector = mesh_selector_dropdown(Rc::clone(self.wrapper));

        let controls = html! {
          <div>
            {mesh_selector}
          </div>
        };

        controls
    }
}

fn mesh_selector_dropdown(wrapper: Rc<RefCell<StateWrapper>>) -> VirtualNode {
    // TODO: Read from state instead of hard coding
    let options = vec!["Cube", "LetterF", "Mesh2", "TexturedCube", "Mesh3", "Mesh1"];

    let options: Vec<VirtualNode> = options
        .into_iter()
        .map(|mesh_name| {
            html! {
                <option name=mesh_name value=mesh_name>
                    {mesh_name}
                </option>
            }
        })
        .collect();

    html! {
      <select
        onchange=move |e: web_sys::Event| {
            let mesh_name: web_sys::HtmlSelectElement = e.target().unwrap().dyn_into().unwrap();
            let mesh_name = mesh_name.value();

            wrapper.borrow_mut().msg(Msg::SetCurrentMesh(mesh_name));
        }
      >
        {options}
      </select>
    }
}
