//! Platform specific functions and traits.

use spin::Mutex;

static GLOBAL_PLATFORM: Mutex<Option<Platform>> = Mutex::new(None);

/// The `Platform` specifies function callbacks that are used in
/// the SBI handler.
///
/// For every function/action that is `None` and the SBI for that
/// functions gets called, will return [`Error::NotSupported`].
pub struct Platform {
    /// Programs the clock for the next event after `stime` (the given argument).
    ///
    /// This method must **not** clear the pending timer interrupt bit.
    pub set_timer: fn(u64),
}

/// Returns a referencet to the Mutex-locked global platform.
pub(crate) fn global() -> &'static Mutex<Option<Platform>> {
    &GLOBAL_PLATFORM
}
