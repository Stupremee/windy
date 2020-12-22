extern "C" {
    static mut __heap_start: u8;
    static mut __heap_size: u8;
}

/// Returns the pointer to the heap start
/// specified by the linker.
pub fn heap_start() -> *mut u8 {
    unsafe { &mut __heap_start }
}

/// Returns the pointer to the heap start
/// specified by the linker.
pub fn heap_size() -> usize {
    unsafe { &mut __heap_size as *mut _ as usize }
}
