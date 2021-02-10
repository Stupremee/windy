//! Memory Allocation APIs.

pub mod buddy;
pub use buddy::BuddyAllocator;

use crate::unit::{self, KIB};
use core::{fmt, ptr::NonNull};
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

/// Return the previous power of two for the given number
pub fn prev_power_of_two(num: usize) -> usize {
    1 << (usize::BITS as usize - num.leading_zeros() as usize - 1)
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
        /// tried to allocate zero pages using `alloc_pages`
        AllocateZeroPages,
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

static PHYS_MEM_ALLOCATOR: GlobalAllocator = GlobalAllocator(Mutex::new(BuddyAllocator::new()));

/// The central allocator that is responsible for allocating physical memory.
pub struct GlobalAllocator(Mutex<BuddyAllocator>);

impl GlobalAllocator {
    /// Adds a single region of memory to this allocator and makes it available for allocation.
    pub unsafe fn add_region(&self, start: NonNull<u8>, end: NonNull<u8>) -> Result<usize> {
        self.0.lock().add_region(start, end)
    }

    /// Allocatge a single page of physmem.
    pub fn alloc(&self) -> Result<NonNull<[u8]>, Error> {
        // order 0 is exactly the page size
        self.0.lock().allocate(0)
    }

    /// Allocatge multiple pages of physical memory.
    pub fn alloc_pages(&self, count: usize) -> Result<NonNull<[u8]>, Error> {
        if count == 0 {
            return Err(Error::AllocateZeroPages);
        }

        // to allocate multiple pages of contigous memory, we
        // allocate the previous power of two, because order `0`,
        // is one page, order `1` two pages, and order `2` four pages.
        //
        // the minus `1` is required, because imagine `count` is two, the previous
        // number of two, would be `2`, but that is not the smallest possible order
        let count = core::cmp::max(1, count - 1);
        self.0.lock().allocate(prev_power_of_two(count))
    }

    /// Return the statistics for this allocator.
    pub fn stats(&self) -> AllocStats {
        self.0.lock().stats()
    }
}

unsafe impl Send for GlobalAllocator {}
unsafe impl Sync for GlobalAllocator {}

/// Return a reference to the global allocator for physical memory.
pub(super) fn allocator() -> &'static GlobalAllocator {
    &PHYS_MEM_ALLOCATOR
}
