use crate::state_wrapper::{Msg, State, StateWrapper};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use virtual_dom_rs::prelude::*;
use wasm_bindgen::JsCast;

pub struct ControlsView<'a> {
    pub wrapper: &'a Rc<RefCell<StateWrapper>>,
}

impl<'a> View for ControlsView<'a> {
    fn render(&self) -> VirtualNode {
        let mesh_selector = mesh_selector_dropdown(Rc::clone(self.wrapper));
        let roughness_slider = roughness_slider(Rc::clone(self.wrapper));
        let metallic_slider = metallic_slider(Rc::clone(self.wrapper));

        let controls = html! {
          <div>
            {mesh_selector}
            {roughness_slider}
            {metallic_slider}
          </div>
        };

        controls
    }
}

fn mesh_selector_dropdown(wrapper: Rc<RefCell<StateWrapper>>) -> VirtualNode {
    // TODO: Read from state instead of hard coding
    let options = vec![
        "Cube",
        "LetterF",
        "Mesh2",
        "TexturedCube",
        "Mesh3",
        "Mesh1",
        "GoldCube",
        "Suzanne",
    ];

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

    let model = &wrapper.borrow().current_model.clone();

    html! {
      <select
        value={model}
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

// FIXME: Re-render whenever store changes ... Need to add a listener ..
// FIXME: Normalize with other slider components
fn roughness_slider(wrapper: Rc<RefCell<StateWrapper>>) -> VirtualNode {
    let roughness = wrapper.borrow().roughness();
    let roughness = format!("{}", roughness);

    html! {
    <div>
        <label>
          Roughness:
          <input
            type="range"
            value=roughness
            min="0.0"
            max="1.0"
            step="0.025"
            oninput=move |e: web_sys::Event| {
                let roughness: web_sys::HtmlInputElement  = e.target().unwrap().dyn_into().unwrap();
                let roughness = roughness.value();
                let roughness = f32::from_str(roughness.as_str()).unwrap();

                wrapper.borrow_mut().msg(Msg::SetRoughness(roughness));
            }
           />
           <span>{ roughness.as_str() }</span>
        </label>
    </div>
    }
}

// FIXME: Re-render whenever store changes ... Need to add a listener ..
// FIXME: Normalize with other slider components
fn metallic_slider(wrapper: Rc<RefCell<StateWrapper>>) -> VirtualNode {
    let metallic = wrapper.borrow().metallic();
    let metallic = format!("{}", metallic);

    html! {
    <div>
        <label>
          Metallic:
          <input
            type="range"
            value=metallic
            min="0.0"
            max="1.0"
            step="1"
            oninput=move |e: web_sys::Event| {
                let metallic: web_sys::HtmlInputElement  = e.target().unwrap().dyn_into().unwrap();
                let metallic = metallic.value();
                let metallic = f32::from_str(metallic.as_str()).unwrap();

                wrapper.borrow_mut().msg(Msg::SetMetallic(metallic));
            }
           />
           <span>{ metallic.as_str() }</span>
        </label>
    </div>
    }
}
