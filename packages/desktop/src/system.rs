use project_core::data::Timestamp;
use project_core::TimeSource;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now_millis() -> Timestamp {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Timestamp(ms)
    }
}
