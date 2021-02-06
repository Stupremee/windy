//! Synchronization primitives.

//use core::num::NonZeroUsize;
//use spinning_top::lock_api;

pub use spinning_top::{Spinlock as Mutex, SpinlockGuard as MutexGuard};

// FIXME: Currently broken because `mhartid` is not readabl

///// A mutex which can be recursively locked by a thread, without deadlocking.
//pub type ReentrantMutex<T> = lock_api::ReentrantMutex<spinning_top::RawSpinlock, GetHartId, T>;

///// Structure to implement `GetThreadId`.
//pub struct GetHartId {
//_priv: (),
//}

//unsafe impl lock_api::GetThreadId for GetHartId {
//const INIT: Self = Self { _priv: () };

//fn nonzero_thread_id(&self) -> NonZeroUsize {
////  we add 1 here because `mhartid` may contain `0` as the hart id.
//unsafe { NonZeroUsize::new_unchecked(crate::csr::mhartid::read() + 1) }
//}
//}
