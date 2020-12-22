//! Implementation of the Memory System for the kernel

use core::ptr::NonNull;

/// The size of a single memory page is 4KiB,
/// this is also the size order-0 in the buddy
/// allocator.
pub const PAGE_SIZE: usize = 1 << 10;

pub mod buddy;
pub mod linked_list;
pub mod slab;

pub use linked_list::LinkedList;

/// Returns the range of the heap defined by the linker script
///
/// Returns `(start, end)` pointers
pub fn heap_range() -> (NonNull<u8>, NonNull<u8>) {
    let start = crate::utils::heap_start();
    // SAFETY
    // The heap values come from the linker, so they must
    // be valid.
    let end = unsafe { start.add(crate::utils::heap_size()) };

    let start = NonNull::new(start as *mut _).expect("Heap start pointer was NULL");
    let end = NonNull::new(end as *mut _).expect("Heap end pointer was NULL");

    (start, end)
}
