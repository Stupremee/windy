use core::{mem, ptr::NonNull};

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

/// Aligns the given pointer to the alignment of `T`.
pub fn align_non_null<P, T>(ptr: NonNull<P>) -> NonNull<P> {
    let ptr = ptr.as_ptr().wrapping_add(mem::align_of::<T>());
    NonNull::new(ptr).expect("aligned pointer of `NonNull` is zero")
}

/// Returns the previous value, that is a power of two.
pub fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * mem::size_of::<usize>() - num.leading_zeros() as usize - 1)
}
