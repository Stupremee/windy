//! Linker symbols

macro_rules! linker_section {
    ($fn:ident, $start:ident, $end:ident) => {
        pub fn $fn() -> (*mut u8, *mut u8) {
            extern "C" {
                static mut $start: Symbol;
                static mut $end: Symbol;
            }

            unsafe { ($start.ptr(), $end.ptr()) }
        }
    };
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

linker_section!(kernel_range, __kernel_start, __kernel_end);
linker_section!(text_range, __text_start, __text_end);
linker_section!(rodata_range, __rodata_start, __rodata_end);
linker_section!(data_range, __data_start, __data_end);
linker_section!(tdata_range, __tdata_start, __tdata_end);
linker_section!(bss_range, __bss_start, __bss_end);
linker_section!(stack_range, __stack_start, __stack_end);
