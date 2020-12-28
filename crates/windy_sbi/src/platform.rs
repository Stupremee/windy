//! Platform specific functions and traits.

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
    /// Defines the number of harts that can be access by the SBI.
    ///
    /// Must be the maximum, exclusie hart id, so that `0..hart_count` is the
    /// range of all available harts.
    pub hart_count: usize,
}
