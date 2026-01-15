use crate::data::Timestamp;

/// System utilities and abstractions used across crates.
///
/// This module exposes a `TimeSource` trait. Concrete implementations
/// (e.g. `SystemTimeSource`) live in platform crates such as `web` and
/// `desktop` so that platform-specific APIs (js_sys / std) are not required
/// in the core crate.
pub trait TimeSource: PartialEq {
    /// Return current Unix timestamp in milliseconds.
    fn now_millis() -> Timestamp;
}
