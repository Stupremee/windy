//! All macros that are used throughout the Kernel.
//!
//! Contains stuff like `print` and `dbg`.

// FIXME: REPLACE WITH ACTUAL `dbg` MACRO
macro_rules! dbg {
    () => {{
        #[allow(unused_unsafe)]
        let _ = unsafe { *(0x10000000 as *mut u8) = '*' as u8 };
    }};
}
