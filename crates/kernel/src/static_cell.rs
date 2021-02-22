use core::cell::UnsafeCell;

/// A cell around a `T`, which implements `Send` + `Sync` and can be
/// access using `unsafe`.
#[repr(transparent)]
pub struct StaticCell<T> {
    inner: UnsafeCell<T>,
}

impl<T> StaticCell<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }

    pub unsafe fn get(&self) -> *mut T {
        self.inner.get()
    }
}

unsafe impl<T> Send for StaticCell<T> {}
unsafe impl<T> Sync for StaticCell<T> {}
