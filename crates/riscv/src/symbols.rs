//! Linker symbols

extern "C" {
    static mut __kernel_start: Symbol;
    static mut __kernel_end: Symbol;
}

/// Helper struct to make handling with Linker Symbols easier.
#[repr(transparent)]
pub struct Symbol(u8);

impl Symbol {
    /// Treats this symbol as a mutable pointer to a byte.
    pub fn ptr(&mut self) -> *mut u8 {
        self as *mut _ as *mut _
    }

    /// Treats this symbol as a value, that is retrieved by
    /// using the value of the address where this symbol points to.
    pub fn value(&self) -> usize {
        self as *const _ as usize
    }
}

/// Returns a `(start, end)` pair of the whole kernel code.
pub fn kernel_range() -> (*mut u8, *mut u8) {
    unsafe { (__kernel_start.ptr(), __kernel_end.ptr()) }
}
