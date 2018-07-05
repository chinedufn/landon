use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use web_apis::performance;

pub struct State {
    pub last_tick_time: SystemTime,
    pub app_start_time: SystemTime,
}

impl State {
    pub fn new() -> State {
        State {
            last_tick_time: State::performance_now_to_system_time(),
            app_start_time: State::performance_now_to_system_time(),
        }
    }

    pub fn performance_now_to_system_time() -> SystemTime {
        let now = performance.now();

        let seconds = (now as u64) / 1_000;
        let nanos = ((now as u32) % 1_000) * 1_000_000;

        UNIX_EPOCH + Duration::new(seconds, nanos)
    }
}
