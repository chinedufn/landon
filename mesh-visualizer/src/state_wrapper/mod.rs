use crate::assets::Assets;
pub use crate::state_wrapper::msg::Msg;
use crate::state_wrapper::msg::Msg::SetRoughness;
pub use crate::state_wrapper::state::State;
use blender_mesh::MaterialInput;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

mod msg;
mod state;

pub struct StateWrapper {
    state: PreventDirectStateMutation,
    assets: Rc<RefCell<Assets>>,
}

impl StateWrapper {
    pub fn new(state: State, assets: Rc<RefCell<Assets>>) -> StateWrapper {
        StateWrapper {
            state: PreventDirectStateMutation(state),
            assets,
        }
    }
}

impl StateWrapper {
    pub fn msg(&mut self, msg: Msg) {
        match &msg {
            Msg::SetCurrentMesh(mesh_name) => {
                match self
                    .assets
                    .borrow()
                    .meshes()
                    .borrow()
                    .get(mesh_name.as_str())
                {
                    Some(mesh) => {
                        if let Some((_name, material)) = mesh.materials().iter().next() {
                            if let MaterialInput::Uniform(roughness) = material.roughness() {
                                self.state.msg(Msg::SetRoughness(*roughness));
                            }

                            if let MaterialInput::Uniform(metallic) = material.metallic() {
                                self.state.msg(Msg::SetRoughness(*metallic));
                            }
                        }

                        self.state.msg(msg)
                    }
                    None => {
                        self.state.msg(msg);
                    }
                };
            }
            _ => self.state.msg(msg),
        }
    }
}

impl Deref for StateWrapper {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

struct PreventDirectStateMutation(State);

impl Deref for PreventDirectStateMutation {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PreventDirectStateMutation {
    fn msg(&mut self, msg: Msg) {
        self.0.msg(msg)
    }
}
