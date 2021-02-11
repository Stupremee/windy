//! Synchronization primitives.

pub use spinning_top::{Spinlock as Mutex, SpinlockGuard as MutexGuard};

// FIXME: Currently broken because `mhartid` is not readable
