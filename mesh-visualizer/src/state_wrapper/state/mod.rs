use crate::state_wrapper::msg::Msg;
use crate::state_wrapper::state::camera::Camera;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use web_sys::window;

mod camera;

pub struct State {
    pub last_tick_time: SystemTime,
    pub app_start_time: SystemTime,
    /// The model that the user is currently viewing in their browser
    pub current_model: String,
    camera: Camera,
}

impl State {
    pub fn new() -> State {
        State {
            last_tick_time: State::performance_now_to_system_time(),
            app_start_time: State::performance_now_to_system_time(),
            current_model: "TexturedCube".to_string(),
            camera: Camera::new(),
        }
    }

    pub fn performance_now_to_system_time() -> SystemTime {
        let now = window().unwrap().performance().unwrap().now();

        let seconds = (now as u64) / 1_000;
        let nanos = ((now as u32) % 1_000) * 1_000_000;

        UNIX_EPOCH + Duration::new(seconds, nanos)
    }
}

impl State {
    pub fn msg(&mut self, msg: Msg) {
        match msg {
            Msg::Zoom(zoom) => {
                self.camera_mut().zoom(zoom);
            }
            Msg::SetCurrentMesh(mesh_name) => self.current_model = mesh_name.to_string(),
        }
    }
}

impl State {
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
