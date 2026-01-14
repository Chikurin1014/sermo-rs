use js_sys::Date;

use project_core::TimeSource;

pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now_millis() -> u64 {
        Date::now() as u64
    }
}
