pub use crate::state_wrapper::msg::Msg;
pub use crate::state_wrapper::state::State;
use std::ops::Deref;

mod msg;
mod state;

pub struct StateWrapper {
    state: PreventDirectStateMutation,
}

impl StateWrapper {
    pub fn new(state: State) -> StateWrapper {
        StateWrapper {
            state: PreventDirectStateMutation(state),
        }
    }
}

impl StateWrapper {
    pub fn msg(&mut self, msg: Msg) {
        self.state.msg(msg);
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
