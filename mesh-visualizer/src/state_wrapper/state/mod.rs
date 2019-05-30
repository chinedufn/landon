use crate::state_wrapper::msg::Msg;
use crate::state_wrapper::state::camera::Camera;
use crate::state_wrapper::state::mouse::Mouse;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use web_sys::window;

mod camera;
mod mouse;

pub struct State {
    pub last_tick_time: SystemTime,
    pub app_start_time: SystemTime,
    /// The model that the user is currently viewing in their browser
    pub current_model: String,
    mouse: Mouse,
    camera: Camera,
    roughness: f32,
    metallic: f32,
}

impl State {
    pub fn new() -> State {
        State {
            last_tick_time: State::performance_now_to_system_time(),
            app_start_time: State::performance_now_to_system_time(),
            current_model: "Suzanne".to_string(),
            mouse: Mouse::default(),
            camera: Camera::new(),
            roughness: 0.5,
            metallic: 0.0,
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
            Msg::SetCurrentMesh(mesh_name) => self.current_model = mesh_name.to_string(),
            Msg::Zoom(zoom) => {
                self.camera_mut().zoom(zoom);
            }
            Msg::MouseDown(x, y) => {
                self.mouse.set_pressed(true);
                self.mouse.set_pos(x, y);
            }
            Msg::MouseUp => {
                self.mouse.set_pressed(false);
            }
            Msg::MouseMove(x, y) => {
                if !self.mouse.get_pressed() {
                    return;
                }

                let (old_x, old_y) = self.mouse.get_pos();

                let x_delta = old_x as i32 - x;
                let y_delta = y - old_y as i32;

                self.camera.orbit_left_right(x_delta as f32 / 50.0);
                self.camera.orbit_up_down(y_delta as f32 / 50.0);

                self.mouse.set_pos(x, y);
            }
            Msg::SetRoughness(roughness) => {
                self.roughness = roughness;
            }
            Msg::SetMetallic(metallic) => {
                self.metallic = metallic;
            }
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

impl State {
    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    pub fn metallic(&self) -> f32 {
        self.metallic
    }
}
