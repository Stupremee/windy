//! Memory Allocation APIs.

pub mod buddy;
pub use buddy::BuddyAllocator;

use crate::unit::{self, KIB};
use core::{alloc::Layout, fmt, ptr::NonNull};
use displaydoc_lite::displaydoc;
use riscv::sync::Mutex;

/// The size of a single page in memory.
///
/// This is also used as the order-0 size inside
/// the buddy allocator.
pub const PAGE_SIZE: usize = 4 * KIB;

/// Result for every memory allocation operation.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Aligns the given `addr` upwards to `align`.
///
/// # Safety
/// Requires `align` to be a power of two.
pub unsafe fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

displaydoc! {
    /// Any error that can happen while allocating or deallocating memory.
    #[derive(Debug)]
    pub enum Error {
        /// tried to add a region to an allocator that was too small.
        RegionTooSmall,
        /// the `end` pointer of a memory region was before the `start` pointer.
        InvalidRegion,
        /// tried to allocate an order that exceeded the maximum order.
        OrderTooLarge,
        /// tried to allocate, but there was no free memory left.
        NoMemoryAvailable,
    }
}

/// Statistics for a memory allocator.
#[derive(Debug, Clone)]
pub struct AllocStats {
    /// The name of the allocator that collected these stat.s
    pub name: &'static str,
    /// The number of size that were allocated.
    pub allocated: usize,
    /// The number of bytes that are left for allocation.
    pub free: usize,
    /// The total number of bytes that this allocator has available for allocation.
    pub total: usize,
}

impl AllocStats {
    /// Create a new [`AllocStats`] instance for the given allocator name.
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            name,
            free: 0,
            allocated: 0,
            total: 0,
        }
    }
}

impl fmt::Display for AllocStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f, "\nAllocated: {}", unit::bytes(self.allocated))?;
        writeln!(f, "Free: {}", unit::bytes(self.free))?;
        writeln!(f, "Total: {}", unit::bytes(self.total))?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f)?;
        Ok(())
    }
}

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator(Mutex::new(BuddyAllocator::new()));

/// The central allocator that is responsible for allocating physical memory.
pub struct GlobalAllocator(Mutex<BuddyAllocator>);

impl GlobalAllocator {
    /// Adds a single region of memory to this allocator and makes it available for allocation.
    pub unsafe fn add_region(&self, start: NonNull<u8>, end: NonNull<u8>) -> Result<usize> {
        self.0.lock().add_region(start, end)
    }

    /// Return the current statistics of this allocator.
    pub fn stats(&self) -> AllocStats {
        self.0.lock().stats()
    }
}

unsafe impl core::alloc::GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::debug!(
            "Allocating block of memory with size {:#X} and alignment {:#X}",
            layout.size(),
            layout.align()
        );

        // buddy allocator always returns page-aligned pointers
        if layout.align() > PAGE_SIZE {
            crate::error!("Tried to allocate memory with alignmnet larger than 8");
            return core::ptr::null_mut();
        }

        let order = buddy::order_for_size(layout.size());
        match self.0.lock().allocate(order) {
            Ok(ptr) => {
                let ptr = ptr.as_mut_ptr();
                assert_eq!(
                    ptr.align_offset(layout.align()),
                    0,
                    "allocator returned unaligned pointer"
                );
                ptr
            }
            Err(err) => {
                crate::warn!("Failed to allocate physical memory: {}", err);
                core::ptr::null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::debug!(
            "Deallocating block at {:#X} of memory with size {:#X} and alignment {:#X}",
            ptr as usize,
            layout.size(),
            layout.align()
        );

        let order = buddy::order_for_size(layout.size());
        self.0.lock().deallocate(NonNull::new_unchecked(ptr), order);
    }
}

/// Returns a reference to the global allocator.
pub fn allocator() -> &'static GlobalAllocator {
    &ALLOCATOR
}

unsafe impl Send for GlobalAllocator {}
unsafe impl Sync for GlobalAllocator {}
