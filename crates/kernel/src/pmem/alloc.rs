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
        /// Tried to create a `NonNull` from a null pointer.
        ///
        /// Mostly just a safety mechanism to avoid UB.
        NullPointer,
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

    /// Deallocate the given page, with the given order.
    pub unsafe fn dealloc(&self, ptr: NonNull<u8>, order: usize) {
        match self.0.lock().deallocate(ptr, order) {
            Ok(()) => {}
            Err(err) => warn!("Failed to deallocate page: {}", err),
        }
    }

    /// Allocatge multiple pages of physical memory.
    pub fn alloc_pages(&self, count: usize) -> Result<NonNull<[u8]>, Error> {
        if count == 0 {
            return Err(Error::AllocateZeroPages);
        }

        // to get the order required to fix `count` pages,
        // we calculate the required size to fit `count` pages,
        // and then get the order for this size
        let total = count * PAGE_SIZE;
        self.0.lock().allocate(buddy::order_for_size(total))
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
