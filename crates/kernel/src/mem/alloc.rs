//! Memory Allocation APIs.

pub mod buddy;
pub use buddy::BuddyAllocator;

use crate::unit::{self, KIB};
use core::fmt;
use displaydoc_lite::displaydoc;

/// The size of a single page in memory.
///
/// This is also used as the order-0 size inside
/// the buddy allocator.
pub const PAGE_SIZE: usize = 4 * KIB;

/// Result for every memory allocation operation.
pub type Result<T, E = Error> = core::result::Result<T, E>;

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
    /// The number of size that were requested by the kernel
    pub requested: usize,
    /// The number of bytes that were actually allocated.
    pub allocated: usize,
    /// The total number of bytes that this allocator has available for allocation.
    pub total: usize,
}

impl AllocStats {
    /// Create a new [`AllocStats`] instance for the given allocator name.
    pub const fn with_name(name: &'static str) -> Self {
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
        writeln!(f, "\nRequested bytes: {}", unit::bytes(self.requested))?;
        writeln!(f, "Allocated bytes:   {}", unit::bytes(self.allocated))?;
        writeln!(f, "Total bytes:       {}", unit::bytes(self.total))?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f)?;
        Ok(())
    }
}

/// Aligns the given `addr` upwards to `align`.
///
/// # Safety
/// Requires `align` to be a power of two.
pub unsafe fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
