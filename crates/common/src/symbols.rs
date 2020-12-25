//! Linker symbols

extern "C" {
    static mut __text_start: Symbol;
    static mut __text_end: Symbol;

    static mut __rodata_start: Symbol;
    static mut __rodata_end: Symbol;

    static mut __data_start: Symbol;
    static mut __data_end: Symbol;

    static mut __bss_start: Symbol;
    static mut __bss_end: Symbol;

    static mut __stack_start: Symbol;
    static mut __stack_end: Symbol;

    static mut __trap_stack: Symbol;

    static mut __heap_start: Symbol;
    static mut __heap_size: Symbol;
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

/// Returns a `(start, end)` pair of the `.text` section.
pub fn text_range() -> (*mut u8, *mut u8) {
    unsafe { (__text_start.ptr(), __text_end.ptr()) }
}

/// Returns a `(start, end)` pair of the `.rodata` section.
pub fn rodata_range() -> (*mut u8, *mut u8) {
    unsafe { (__rodata_start.ptr(), __rodata_end.ptr()) }
}

/// Returns a `(start, end)` pair of the `.data` section.
pub fn data_range() -> (*mut u8, *mut u8) {
    unsafe { (__data_start.ptr(), __data_end.ptr()) }
}

/// Returns a `(start, end)` pair of the `.bss` section.
pub fn bss_range() -> (*mut u8, *mut u8) {
    unsafe { (__bss_start.ptr(), __bss_end.ptr()) }
}

/// Returns a pointer to the stack that will be used for trap handling
/// and trap frames.
pub fn trap_stack() -> *mut u8 {
    unsafe { __trap_stack.ptr() }
}

/// Returns a pointer to the start of the heap.
pub fn heap_start() -> *mut u8 {
    unsafe { __heap_start.ptr() }
}

/// Returns the size of the heap.
pub fn heap_size() -> usize {
    unsafe { __heap_size.value() }
}
