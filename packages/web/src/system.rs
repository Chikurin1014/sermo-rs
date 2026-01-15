use js_sys::Date;

use project_core::{TimeSource, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now_millis() -> Timestamp {
        Timestamp(Date::now() as u64)
    }
}
