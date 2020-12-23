//! Implementation of the Memory System for the kernel

use core::{fmt, ptr::NonNull};

/// The size of a single memory page is 4KiB,
/// this is also the size order-0 in the buddy
/// allocator.
pub const PAGE_SIZE: usize = 1 << 12;

pub mod buddy;
pub mod linked_list;
pub mod slab;

pub use linked_list::LinkedList;

/// Statistics of any kind of allocator.
#[derive(Debug, Clone)]
pub struct AllocStats {
    /// The name of the allocator these stats belong to.
    pub name: &'static str,
    /// The bytes that were actuallly requested by the user.
    pub requested: usize,
    /// The bytes that were actually allocated.
    pub allocated: usize,
    /// The total number of bytes this allocator can reach.
    pub total: usize,
}

impl AllocStats {
    /// Create a new [`AllocStats`] instance for the given allocator name.
    pub fn with_name(name: &'static str) -> Self {
        Self {
            name,
            requested: 0,
            allocated: 0,
            total: 0,
        }
    }
}

impl fmt::Display for AllocStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f, "\nRequested bytes: 0x{:x}", self.requested)?;
        writeln!(f, "Allocated bytes: 0x{:x}", self.allocated)?;
        writeln!(f, "Total bytes:     0x{:x}", self.total)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f)?;
        Ok(())
    }
}

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
